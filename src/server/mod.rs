//! # Servidor do Compositor
//!
//! Módulo principal que gerencia o loop de renderização e processa
//! mensagens IPC de clientes (aplicações).

use crate::input::InputManager;
use crate::render::RenderEngine;
use alloc::vec::Vec;
use gfx_types::Size;
use redpowder::event::{event_type, InputEvent};
use redpowder::ipc::{Port, SharedMemory};
use redpowder::syscall::SysResult;
use redpowder::window::{
    opcodes, CommitBufferRequest, CreateWindowRequest, WindowCreatedResponse, COMPOSITOR_PORT,
    MAX_MSG_SIZE,
};

// ============================================================================
// CONSTANTES
// ============================================================================

const FRAME_INTERVAL_MS: u64 = 16;
const STATS_LOG_INTERVAL: u64 = 300;

// ============================================================================
// ESTRUTURAS AUXILIARES
// ============================================================================

struct ClientPort {
    window_id: u32,
    port: Port,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct InputUpdateRequest {
    pub op: u32,
    pub event_type: u32,
    pub key_code: u32,
    pub key_pressed: u32,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_buttons: u32,
}

// ============================================================================
// SERVIDOR
// ============================================================================

pub struct Server {
    port: Port,
    render_engine: RenderEngine,
    input: InputManager,
    running: bool,
    frame_count: u64,
    client_ports: Vec<ClientPort>,
    focused_window: Option<u32>,
    last_mouse_buttons: u32,
    mouse_x: i32,
    mouse_y: i32,
    dragging_window: Option<u32>,
    drag_off_x: i32,
    drag_off_y: i32,
}

impl Server {
    pub fn new() -> SysResult<Self> {
        redpowder::println!("[Server] Inicializando...");

        let port = Port::create(COMPOSITOR_PORT, 128)?;
        redpowder::println!("[Server] Porta '{}' criada", COMPOSITOR_PORT);

        let display_info = redpowder::graphics::get_framebuffer_info()?;
        redpowder::println!(
            "[Server] Display: {}x{}",
            display_info.width,
            display_info.height
        );

        let gfx_display_info = gfx_types::DisplayInfo {
            id: 0,
            width: display_info.width,
            height: display_info.height,
            refresh_rate_mhz: 60000,
            format: gfx_types::PixelFormat::ARGB8888,
            stride: display_info.stride * 4,
        };

        let render_engine = RenderEngine::new(gfx_display_info);

        Ok(Self {
            port,
            render_engine,
            input: InputManager::new(),
            running: true,
            frame_count: 0,
            client_ports: Vec::new(),
            focused_window: None,
            last_mouse_buttons: 0,
            mouse_x: 100,
            mouse_y: 100,
            dragging_window: None,
            drag_off_x: 0,
            drag_off_y: 0,
        })
    }

    pub fn run(&mut self) -> SysResult<()> {
        let mut msg_buf = [0u8; MAX_MSG_SIZE];
        let mut loop_count = 0u64;

        while self.running {
            loop_count += 1;

            if loop_count % 600 == 0 {
                let (_, win_count) = self.render_engine.stats();
                redpowder::println!(
                    "[Compositor] Loop ativo, {} janelas, foco: {:?}",
                    win_count,
                    self.focused_window
                );
            }

            // 1. Processar mensagens IPC
            self.process_messages(&mut msg_buf)?;

            // 2. Renderizar frame com cursor
            self.render_engine
                .render_with_cursor(self.mouse_x, self.mouse_y)?;

            // 3. Estabilizar Framerate
            let _ = redpowder::time::sleep(FRAME_INTERVAL_MS);
        }

        Ok(())
    }

    fn process_messages(&mut self, buf: &mut [u8; MAX_MSG_SIZE]) -> SysResult<()> {
        while let Ok(size) = self.port.recv(buf, 0) {
            if size > 0 {
                self.handle_message(&buf[..size])?;
            } else {
                break; // Fila vazia
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
            opcodes::CREATE_WINDOW => self.handle_create_window(data),
            opcodes::COMMIT_BUFFER => self.handle_commit_buffer(data),
            opcodes::DESTROY_WINDOW => self.handle_destroy_window(data),
            opcodes::INPUT_UPDATE => self.handle_input_update(data),
            _ => {
                redpowder::println!("[Server] Opcode desconhecido: {:#x}", opcode);
                Ok(())
            }
        }
    }

    fn handle_input_update(&mut self, data: &[u8]) -> SysResult<()> {
        if data.len() < core::mem::size_of::<InputUpdateRequest>() {
            redpowder::println!("[Server] Erro: INPUT_UPDATE muito curto ({})", data.len());
            return Ok(());
        }

        let req = unsafe { &*(data.as_ptr() as *const InputUpdateRequest) };

        // 1. Atualizar estado interno
        self.input.update_from_service(
            req.event_type,
            req.key_code,
            req.key_pressed,
            req.mouse_x,
            req.mouse_y,
            req.mouse_buttons,
        );

        // 2. Se for teclado, enviar para janela focada
        if req.event_type == 1 {
            // Key
            if let Some(target_id) = self.focused_window {
                redpowder::println!(
                    "[Server] Dispatching Key {} to window {}",
                    req.key_code,
                    target_id
                );
                self.dispatch_key_event(target_id, req.key_code, req.key_pressed == 1);
            } else {
                redpowder::println!("[Server] Key {} ignored (no focus)", req.key_code);
            }
        }

        // 3. Se for mouse, processar foco e eventos
        if req.event_type == 2 {
            // Atualizar posição do cursor
            self.mouse_x = req.mouse_x;
            self.mouse_y = req.mouse_y;

            let buttons = req.mouse_buttons;
            let left_button_now = (buttons & 0x01) != 0;
            let left_button_was = (self.last_mouse_buttons & 0x01) != 0;

            // Detectar click (transição de não pressionado para pressionado)
            if left_button_now && !left_button_was {
                // Encontrar janela sob cursor
                if let Some(window_id) =
                    self.render_engine.window_at_point(req.mouse_x, req.mouse_y)
                {
                    if self.focused_window != Some(window_id) {
                        redpowder::println!(
                            "[Server] Click detected! Focus changed to window {}",
                            window_id
                        );
                        self.focused_window = Some(window_id);
                    }

                    // Tentar iniciar arraste se clicar na barra de título (topo da janela)
                    // Janelas normais tem ~30-40px de barra de título
                    if let Some(win) = self.render_engine.get_window(window_id) {
                        let win_rect = win.rect();
                        let title_height = 40;
                        if req.mouse_y >= win_rect.y && req.mouse_y < win_rect.y + title_height {
                            redpowder::println!("[Server] Drag START window {}", window_id);
                            // Só arrastar se não for a Window 1 (geralmente o Desktop/Wallpaper)
                            if window_id != 1 {
                                self.dragging_window = Some(window_id);
                                self.drag_off_x = req.mouse_x - win_rect.x;
                                self.drag_off_y = req.mouse_y - win_rect.y;
                            }
                        } else {
                            redpowder::println!(
                                "[Server] Click in window {} but NOT in title bar (y={}, win_y={})",
                                window_id,
                                req.mouse_y,
                                win_rect.y
                            );
                        }
                    }

                    // Enviar evento de click para a janela
                    self.dispatch_mouse_event(window_id, req.mouse_x, req.mouse_y, buttons, true);
                }
            }

            // 3.1. Processar Arraste (se já estiver arrastando)
            if let Some(win_id) = self.dragging_window {
                if left_button_now {
                    let new_x = req.mouse_x - self.drag_off_x;
                    let new_y = req.mouse_y - self.drag_off_y;
                    self.render_engine.move_window(win_id, new_x, new_y);
                } else {
                    self.dragging_window = None;
                }
            }

            // Detectar release
            if !left_button_now && left_button_was {
                if let Some(focused) = self.focused_window {
                    self.dispatch_mouse_event(focused, req.mouse_x, req.mouse_y, buttons, false);
                }
                self.dragging_window = None;
            }

            self.last_mouse_buttons = buttons;
        }

        Ok(())
    }

    fn dispatch_key_event(&mut self, window_id: u32, key_code: u32, pressed: bool) {
        let event = InputEvent {
            op: opcodes::EVENT_INPUT,
            event_type: if pressed {
                event_type::KEY_DOWN
            } else {
                event_type::KEY_UP
            },
            param1: key_code,
            param2: 0, // Modifiers no futuro
        };

        let bytes = unsafe {
            core::slice::from_raw_parts(
                &event as *const _ as *const u8,
                core::mem::size_of::<InputEvent>(),
            )
        };

        if let Some(client) = self.client_ports.iter().find(|c| c.window_id == window_id) {
            let res = client.port.send(bytes, 0);
            if let Err(e) = res {
                redpowder::println!(
                    "[Server] Erro ao enviar evento para janela {}: {:?}",
                    window_id,
                    e
                );
            }
        } else {
            redpowder::println!(
                "[Server] Erro: Nao encontrei porta para janela {}",
                window_id
            );
        }
    }

    fn dispatch_mouse_event(
        &mut self,
        window_id: u32,
        x: i32,
        y: i32,
        buttons: u32,
        pressed: bool,
    ) {
        let event = InputEvent {
            op: opcodes::EVENT_INPUT,
            event_type: if pressed {
                event_type::MOUSE_DOWN
            } else {
                event_type::MOUSE_UP
            },
            param1: x as u32,
            param2: ((y as u32) << 16) | (buttons & 0xFFFF),
        };

        let bytes = unsafe {
            core::slice::from_raw_parts(
                &event as *const _ as *const u8,
                core::mem::size_of::<InputEvent>(),
            )
        };

        if let Some(client) = self.client_ports.iter().find(|c| c.window_id == window_id) {
            let _ = client.port.send(bytes, 0);
        }
    }

    fn handle_create_window(&mut self, data: &[u8]) -> SysResult<()> {
        let req = unsafe { &*(data.as_ptr() as *const CreateWindowRequest) };

        let buffer_size = (req.width * req.height * 4) as usize;
        let mut shm = match SharedMemory::create(buffer_size) {
            Ok(s) => s,
            Err(e) => {
                redpowder::println!("[Server] Falha ao criar SHM: {:?}", e);
                return Ok(());
            }
        };

        let pixel_count = (req.width * req.height) as usize;
        let pixels =
            unsafe { core::slice::from_raw_parts_mut(shm.as_mut_ptr() as *mut u32, pixel_count) };
        for pixel in pixels.iter_mut() {
            *pixel = 0xFF000000;
        }

        let shm_id = shm.id();
        let size = gfx_types::Size::new(req.width, req.height);
        let window_id = self.render_engine.create_window(size, shm);
        self.render_engine
            .move_window(window_id, req.x as i32, req.y as i32);

        // Focar na janela criada (simples)
        self.focused_window = Some(window_id);
        redpowder::println!("[Server] Janela {} criada, ganhando foco", window_id);

        let name_len = req
            .reply_port
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(req.reply_port.len());
        if let Ok(port_name) = core::str::from_utf8(&req.reply_port[..name_len]) {
            redpowder::println!("[Server] Conectando a porta de resposta: '{}'", port_name);

            // Retry com delay - a porta pode não estar pronta imediatamente
            let mut reply_port_opt = None;
            for attempt in 0..10 {
                match Port::connect(port_name) {
                    Ok(p) => {
                        reply_port_opt = Some(p);
                        break;
                    }
                    Err(_) => {
                        if attempt < 9 {
                            redpowder::time::sleep(10); // 10ms entre tentativas
                        }
                    }
                }
            }

            if let Some(reply_port) = reply_port_opt {
                // Enviar resposta ANTES de mover a porta
                let response = WindowCreatedResponse {
                    op: opcodes::WINDOW_CREATED,
                    window_id,
                    shm_handle: shm_id.0,
                    buffer_size: buffer_size as u64,
                };

                let resp_bytes = unsafe {
                    core::slice::from_raw_parts(
                        &response as *const _ as *const u8,
                        core::mem::size_of::<WindowCreatedResponse>(),
                    )
                };
                let _ = reply_port.send(resp_bytes, 0);

                // Agora movemos a porta para client_ports (sem clonar)
                self.client_ports.push(ClientPort {
                    window_id,
                    port: reply_port,
                });

                redpowder::println!(
                    "[Server] Conectado e resposta enviada para janela {}",
                    window_id
                );
            } else {
                redpowder::println!(
                    "[Server] Falha ao conectar na porta de resposta '{}' após 10 tentativas",
                    port_name
                );
            }
        }

        let display_size = self.render_engine.size();
        if req.width == display_size.width
            && req.height == display_size.height
            && req.x == 0
            && req.y == 0
        {
            self.render_engine
                .set_window_layer(window_id, gfx_types::LayerType::Background);
            // Background não recebe foco de teclado
            self.focused_window = None;
        }

        Ok(())
    }

    fn handle_commit_buffer(&mut self, data: &[u8]) -> SysResult<()> {
        let req = unsafe { &*(data.as_ptr() as *const CommitBufferRequest) };
        self.render_engine.mark_window_has_content(req.window_id);
        self.render_engine.mark_damage(req.window_id);
        Ok(())
    }

    fn handle_destroy_window(&mut self, _data: &[u8]) -> SysResult<()> {
        Ok(())
    }
}
