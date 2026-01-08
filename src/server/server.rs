//! # Server
//!
//! Servidor principal do compositor Firefly.

use alloc::vec::Vec;
use gfx_types::display::DisplayInfo;
use gfx_types::window::LayerType;
use redpowder::graphics::get_info;
use redpowder::ipc::Port;
use redpowder::syscall::SysResult;
use redpowder::window::{
    lifecycle_events, opcodes, DestroyWindowRequest, RegisterTaskbarRequest, WindowOpRequest,
    COMPOSITOR_PORT, MAX_MSG_SIZE,
};

use crate::input::InputManager;
use crate::render::RenderEngine;

use super::dispatch::{dispatch_key_event, dispatch_mouse_event, send_lifecycle_event};
use super::handlers;
use super::protocol::{ClientPort, InputUpdateRequest};
use super::state::{ClickState, DragState, MouseState};

// =============================================================================
// CONSTANTES
// =============================================================================

/// Intervalo entre frames (ms) - ~60 FPS.
const FRAME_INTERVAL_MS: u64 = 16;

// =============================================================================
// SERVER
// =============================================================================

/// Servidor principal do compositor Firefly.
pub struct Server {
    /// Porta IPC para receber requisições.
    port: Port,
    /// Motor de renderização.
    render_engine: RenderEngine,
    /// Gerenciador de input.
    input: InputManager,
    /// Servidor está rodando.
    running: bool,
    /// Contador de frames.
    frame_count: u64,
    /// Portas de clientes conectados.
    client_ports: Vec<ClientPort>,
    /// Janela com foco.
    focused_window: Option<u32>,
    /// Estado do mouse.
    mouse: MouseState,
    /// Estado de arraste.
    drag: DragState,
    /// Estado de click.
    click: ClickState,
    /// Porta da taskbar.
    taskbar_port: Option<Port>,
}

impl Server {
    /// Cria novo servidor.
    pub fn new() -> SysResult<Self> {
        // Use write_str direto para garantir que o log aparece (sem alocação)
        let _ = redpowder::console::write_str("[Firefly] Server::new() ENTRY\n");

        // 1. Criar porta IPC
        let _ = redpowder::console::write_str("[Firefly] Criando porta IPC...\n");
        let port = Port::create(COMPOSITOR_PORT, 128)?;
        let _ = redpowder::console::write_str("[Firefly] Porta IPC criada OK\n");

        // 2. Obter informações do display
        let _ = redpowder::console::write_str("[Firefly] Obtendo info display...\n");
        let fb_info = get_info()?;
        let _ = redpowder::console::write_str("[Firefly] Display info OK\n");
        redpowder::println!(
            "[Firefly] Display: {}x{} stride={}",
            fb_info.width,
            fb_info.height,
            fb_info.stride
        );

        // 3. Criar DisplayInfo para gfx_types
        let display_info = DisplayInfo {
            id: 0,
            width: fb_info.width,
            height: fb_info.height,
            refresh_rate_mhz: 60_000,
            format: gfx_types::color::PixelFormat::ARGB8888,
            stride: fb_info.stride * 4,
        };

        // 4. Criar motor de renderização
        let render_engine = RenderEngine::new(display_info);

        Ok(Self {
            port,
            render_engine,
            input: InputManager::new(),
            running: true,
            frame_count: 0,
            client_ports: Vec::new(),
            focused_window: None,
            mouse: MouseState::new(),
            drag: DragState::new(),
            click: ClickState::new(),
            taskbar_port: None,
        })
    }

    /// Executa o loop principal do compositor.
    pub fn run(&mut self) -> SysResult<()> {
        let mut msg_buf = [0u8; MAX_MSG_SIZE];
        let mut loop_count = 0u64;

        redpowder::println!("[Firefly] Entrando no loop principal");

        while self.running {
            loop_count += 1;

            // Log periódico
            if loop_count % 600 == 0 {
                let (_, win_count) = self.render_engine.stats();
                redpowder::println!(
                    "[Firefly] Loop {}, {} janelas, foco={:?}",
                    loop_count,
                    win_count,
                    self.focused_window
                );
            }

            // 1. Processar mensagens IPC
            self.process_messages(&mut msg_buf)?;

            // 2. Renderizar frame
            self.render_engine.render(self.mouse.x, self.mouse.y)?;
            self.frame_count += 1;

            // 3. Estabilizar framerate
            let _ = redpowder::time::sleep(FRAME_INTERVAL_MS);
        }

        Ok(())
    }

    // =========================================================================
    // PROCESSAMENTO DE MENSAGENS
    // =========================================================================

    fn process_messages(&mut self, buf: &mut [u8; MAX_MSG_SIZE]) -> SysResult<()> {
        while let Ok(size) = self.port.recv(buf, 0) {
            if size > 0 {
                self.handle_message(&buf[..size])?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn handle_message(&mut self, data: &[u8]) -> SysResult<()> {
        if data.len() < 4 {
            return Ok(());
        }

        let opcode = unsafe { *(data.as_ptr() as *const u32) };

        match opcode {
            opcodes::CREATE_WINDOW => {
                let (window_id, layer) = handlers::handle_create_window(
                    &mut self.render_engine,
                    &mut self.client_ports,
                    self.taskbar_port.as_ref(),
                    data,
                )?;

                // Focar (se não for background)
                if layer != LayerType::Background {
                    self.focused_window = Some(window_id);
                    self.render_engine.set_focus(Some(window_id));
                }
            }
            opcodes::COMMIT_BUFFER => {
                handlers::handle_commit_buffer(&mut self.render_engine, data);
            }
            opcodes::DESTROY_WINDOW => {
                let req = unsafe { &*(data.as_ptr() as *const DestroyWindowRequest) };
                if self.focused_window == Some(req.window_id) {
                    self.focused_window = None;
                    self.render_engine.set_focus(None);
                }
                handlers::handle_destroy_window(
                    &mut self.render_engine,
                    &mut self.client_ports,
                    self.taskbar_port.as_ref(),
                    req.window_id,
                );
            }
            opcodes::INPUT_UPDATE => {
                self.handle_input_update(data)?;
            }
            opcodes::MINIMIZE_WINDOW => {
                let req = unsafe { &*(data.as_ptr() as *const WindowOpRequest) };
                handlers::handle_minimize_window(
                    &mut self.render_engine,
                    self.taskbar_port.as_ref(),
                    req.window_id,
                );
            }
            opcodes::RESTORE_WINDOW => {
                let req = unsafe { &*(data.as_ptr() as *const WindowOpRequest) };
                if let Some(window_id) = handlers::handle_restore_window(
                    &mut self.render_engine,
                    self.taskbar_port.as_ref(),
                    req.window_id,
                ) {
                    self.focused_window = Some(window_id);
                    self.render_engine.set_focus(Some(window_id));
                }
            }
            opcodes::REGISTER_TASKBAR => {
                let req = unsafe { &*(data.as_ptr() as *const RegisterTaskbarRequest) };
                if let Some(port) = handlers::handle_register_taskbar(req) {
                    self.taskbar_port = Some(port);
                }
            }
            _ => {
                redpowder::println!("[Firefly] Opcode desconhecido: {:#x}", opcode);
            }
        }

        Ok(())
    }

    // =========================================================================
    // INPUT
    // =========================================================================

    fn handle_input_update(&mut self, data: &[u8]) -> SysResult<()> {
        if data.len() < core::mem::size_of::<InputUpdateRequest>() {
            return Ok(());
        }

        let req = unsafe { &*(data.as_ptr() as *const InputUpdateRequest) };

        // Atualizar estado interno
        self.input.update_from_service(
            req.event_type,
            req.key_code,
            req.key_pressed,
            req.mouse_x,
            req.mouse_y,
            req.mouse_buttons,
        );

        // Processar teclado
        if req.event_type == 1 {
            if let Some(target_id) = self.focused_window {
                dispatch_key_event(
                    &self.client_ports,
                    target_id,
                    req.key_code,
                    req.key_pressed == 1,
                );
            }
        }

        // Processar mouse
        if req.event_type == 2 {
            self.mouse.update(req.mouse_x, req.mouse_y);
            self.process_mouse_input(req.mouse_buttons)?;
        }

        Ok(())
    }

    fn process_mouse_input(&mut self, buttons: u32) -> SysResult<()> {
        let x = self.mouse.x;
        let y = self.mouse.y;

        // Click (press)
        if self.mouse.left_just_pressed(buttons) {
            self.handle_mouse_click(x, y, buttons)?;
        }

        // Drag
        if let Some(win_id) = self.drag.window_id {
            if self.mouse.left_pressed(buttons) {
                let new_x = x - self.drag.offset_x;
                let new_y = y - self.drag.offset_y;
                self.render_engine.move_window(win_id, new_x, new_y);
                self.render_engine.full_screen_damage();
            } else {
                self.drag.stop();
            }
        }

        // Release
        if self.mouse.left_just_released(buttons) {
            if let Some(focused) = self.focused_window {
                let (rel_x, rel_y) = self.get_relative_coords(focused, x, y);
                dispatch_mouse_event(&self.client_ports, focused, rel_x, rel_y, buttons, false);
            }
            self.drag.stop();
        }

        self.mouse.save_buttons(buttons);
        Ok(())
    }

    fn handle_mouse_click(&mut self, x: i32, y: i32, buttons: u32) -> SysResult<()> {
        let window_id = match self.render_engine.window_at_point(x, y) {
            Some(id) => id,
            None => return Ok(()),
        };

        // Atualizar foco
        if self.focused_window != Some(window_id) {
            self.focused_window = Some(window_id);
            self.render_engine.set_focus(Some(window_id));

            if let Some(win) = self.render_engine.get_window(window_id) {
                let title = win.title.clone();
                send_lifecycle_event(
                    self.taskbar_port.as_ref(),
                    lifecycle_events::FOCUSED,
                    window_id,
                    &title,
                );
            }

            // Trazer para frente (apenas janelas normais)
            if let Some(win) = self.render_engine.get_window(window_id) {
                if win.layer == LayerType::Normal {
                    self.render_engine.bring_to_front(window_id);
                }
            }
        }

        // Dispatch click
        let (rel_x, rel_y) = self.get_relative_coords(window_id, x, y);
        dispatch_mouse_event(&self.client_ports, window_id, rel_x, rel_y, buttons, true);

        // Verificar click na title bar
        self.handle_titlebar_click(window_id, x, y)?;

        Ok(())
    }

    fn handle_titlebar_click(&mut self, window_id: u32, x: i32, y: i32) -> SysResult<()> {
        let (rect, has_decorations, layer) = {
            let win = match self.render_engine.get_window(window_id) {
                Some(w) => w,
                None => return Ok(()),
            };
            (win.rect(), win.has_decorations(), win.layer)
        };

        if !has_decorations || layer == LayerType::Background {
            return Ok(());
        }

        let rel_x = x - rect.x;
        let rel_y = y - rect.y;

        // Title bar (24px height)
        if rel_y >= 0 && rel_y < 24 {
            let w = rect.width as i32;
            let btn_size = 20;
            let close_x = w - btn_size - 2;
            let min_x = w - (btn_size * 2) - 6;

            if rel_x >= close_x && rel_x < close_x + btn_size {
                // Close
                if self.focused_window == Some(window_id) {
                    self.focused_window = None;
                    self.render_engine.set_focus(None);
                }
                handlers::handle_destroy_window(
                    &mut self.render_engine,
                    &mut self.client_ports,
                    self.taskbar_port.as_ref(),
                    window_id,
                );
            } else if rel_x >= min_x && rel_x < min_x + btn_size {
                // Minimize
                handlers::handle_minimize_window(
                    &mut self.render_engine,
                    self.taskbar_port.as_ref(),
                    window_id,
                );
            } else {
                // Title bar drag ou double-click
                if self.click.is_double_click(window_id, self.frame_count) {
                    // Maximize/Restore
                    let screen_size = self.render_engine.size();
                    if let Some(win) = self.render_engine.get_window_mut(window_id) {
                        if win.state == gfx_types::window::WindowState::Maximized {
                            win.restore();
                        } else {
                            win.maximize(screen_size);
                        }
                        self.render_engine.full_screen_damage();
                    }
                    self.click.clear();
                } else {
                    // Start drag
                    self.drag.start(window_id, rel_x, rel_y);
                    self.click.register(window_id, self.frame_count);
                }
            }
        }

        Ok(())
    }

    fn get_relative_coords(&self, window_id: u32, x: i32, y: i32) -> (i32, i32) {
        if let Some(win) = self.render_engine.get_window(window_id) {
            let local = win.to_local(x, y);
            (local.x, local.y)
        } else {
            (x, y)
        }
    }
}
