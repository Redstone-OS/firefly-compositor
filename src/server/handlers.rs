//! # Message Handlers
//!
//! Handlers para mensagens IPC.

use alloc::string::ToString;
use alloc::vec::Vec;
use gfx_types::geometry::Size;
use gfx_types::window::{LayerType, WindowFlags};
use redpowder::ipc::{Port, SharedMemory};
use redpowder::syscall::SysResult;
use redpowder::window::{
    lifecycle_events, opcodes, CommitBufferRequest, CreateWindowRequest, RegisterTaskbarRequest,
    WindowCreatedResponse,
};

use crate::render::RenderEngine;

use super::dispatch::send_lifecycle_event;
use super::protocol::ClientPort;

// =============================================================================
// CREATE WINDOW
// =============================================================================

/// Handler para CREATE_WINDOW.
pub fn handle_create_window(
    render_engine: &mut RenderEngine,
    client_ports: &mut Vec<ClientPort>,
    taskbar_port: Option<&Port>,
    data: &[u8],
) -> SysResult<(u32, LayerType)> {
    let req = unsafe { &*(data.as_ptr() as *const CreateWindowRequest) };

    // 1. Criar memória compartilhada
    let buffer_size = (req.width * req.height * 4) as usize;
    let mut shm = SharedMemory::create(buffer_size)?;

    // 2. Inicializar buffer com preto
    let pixels = unsafe {
        core::slice::from_raw_parts_mut(
            shm.as_mut_ptr() as *mut u32,
            (req.width * req.height) as usize,
        )
    };
    pixels.fill(0xFF000000);

    let shm_id = shm.id();
    let size = Size::new(req.width, req.height);

    // 3. Determinar camada baseada em flags
    let flags = WindowFlags::from_bits(req.flags);
    let layer = determine_layer(&flags, req.y);

    // 4. Extrair título
    let title_len = req
        .title
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(req.title.len());
    let title = core::str::from_utf8(&req.title[..title_len])
        .unwrap_or("Untitled")
        .to_string();

    // 5. Criar janela
    let window_id = render_engine.create_window(size, shm, layer, title.clone());

    // 6. Posicionar
    render_engine.move_window(window_id, req.x as i32, req.y as i32);

    // 7. Aplicar flags
    if let Some(win) = render_engine.get_window_mut(window_id) {
        win.flags = flags;
    }

    // 8. Conectar porta de resposta
    let name_len = req
        .reply_port
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(req.reply_port.len());
    if let Ok(port_name) = core::str::from_utf8(&req.reply_port[..name_len]) {
        connect_and_respond(client_ports, port_name, window_id, shm_id.0, buffer_size);
    }

    // 9. Notificar taskbar
    send_lifecycle_event(taskbar_port, lifecycle_events::CREATED, window_id, &title);

    redpowder::println!(
        "[Firefly] Janela {} criada: {}x{} layer={:?} '{}'",
        window_id,
        req.width,
        req.height,
        layer,
        title
    );

    Ok((window_id, layer))
}

/// Determina a camada baseada nas flags.
fn determine_layer(flags: &WindowFlags, y: u32) -> LayerType {
    if flags.has(WindowFlags::OVERLAY) {
        LayerType::Overlay
    } else if flags.has(WindowFlags::BACKGROUND) {
        LayerType::Background
    } else if flags.has(WindowFlags::BORDERLESS) && y == 0 {
        LayerType::Panel
    } else {
        LayerType::Normal
    }
}

/// Conecta à porta de resposta e envia response.
fn connect_and_respond(
    client_ports: &mut Vec<ClientPort>,
    port_name: &str,
    window_id: u32,
    shm_handle: u64,
    buffer_size: usize,
) {
    for attempt in 0..10 {
        match Port::connect(port_name) {
            Ok(reply_port) => {
                let response = WindowCreatedResponse {
                    op: opcodes::WINDOW_CREATED,
                    window_id,
                    shm_handle,
                    buffer_size: buffer_size as u64,
                };

                let resp_bytes = unsafe {
                    core::slice::from_raw_parts(
                        &response as *const _ as *const u8,
                        core::mem::size_of::<WindowCreatedResponse>(),
                    )
                };
                let _ = reply_port.send(resp_bytes, 0);

                client_ports.push(ClientPort {
                    window_id,
                    port: reply_port,
                });
                break;
            }
            Err(_) if attempt < 9 => {
                let _ = redpowder::time::sleep(10);
            }
            Err(e) => {
                redpowder::println!("[Firefly] Falha ao conectar porta: {:?}", e);
            }
        }
    }
}

// =============================================================================
// DESTROY WINDOW
// =============================================================================

/// Handler para DESTROY_WINDOW.
pub fn handle_destroy_window(
    render_engine: &mut RenderEngine,
    client_ports: &mut Vec<ClientPort>,
    taskbar_port: Option<&Port>,
    window_id: u32,
) {
    redpowder::println!("[Firefly] Destruindo janela {}", window_id);

    client_ports.retain(|c| c.window_id != window_id);
    send_lifecycle_event(taskbar_port, lifecycle_events::DESTROYED, window_id, "");
    render_engine.destroy_window(window_id);
    render_engine.full_screen_damage();
}

// =============================================================================
// COMMIT BUFFER
// =============================================================================

/// Handler para COMMIT_BUFFER.
pub fn handle_commit_buffer(render_engine: &mut RenderEngine, data: &[u8]) {
    let req = unsafe { &*(data.as_ptr() as *const CommitBufferRequest) };
    render_engine.mark_window_has_content(req.window_id);
    render_engine.mark_damage(req.window_id);
}

// =============================================================================
// MINIMIZE/RESTORE WINDOW
// =============================================================================

/// Handler para MINIMIZE_WINDOW.
pub fn handle_minimize_window(
    render_engine: &mut RenderEngine,
    taskbar_port: Option<&Port>,
    window_id: u32,
) {
    if let Some(win) = render_engine.get_window_mut(window_id) {
        win.minimize();
        let title = win.title.clone();
        send_lifecycle_event(taskbar_port, lifecycle_events::MINIMIZED, window_id, &title);
        render_engine.full_screen_damage();
        redpowder::println!("[Firefly] Janela {} minimizada", window_id);
    }
}

/// Handler para RESTORE_WINDOW.
pub fn handle_restore_window(
    render_engine: &mut RenderEngine,
    taskbar_port: Option<&Port>,
    window_id: u32,
) -> Option<u32> {
    if let Some(win) = render_engine.get_window_mut(window_id) {
        win.restore();
        let title = win.title.clone();
        send_lifecycle_event(taskbar_port, lifecycle_events::RESTORED, window_id, &title);
        render_engine.full_screen_damage();
        render_engine.bring_to_front(window_id);
        redpowder::println!("[Firefly] Janela {} restaurada", window_id);
        return Some(window_id);
    }
    None
}

// =============================================================================
// REGISTER TASKBAR
// =============================================================================

/// Handler para REGISTER_TASKBAR.
pub fn handle_register_taskbar(req: &RegisterTaskbarRequest) -> Option<Port> {
    let name_str = core::str::from_utf8(&req.listener_port)
        .unwrap_or("")
        .trim_matches(char::from(0));

    if !name_str.is_empty() {
        match Port::connect(name_str) {
            Ok(p) => {
                redpowder::println!("[Firefly] Taskbar registrada: '{}'", name_str);
                return Some(p);
            }
            Err(e) => {
                redpowder::println!("[Firefly] Falha ao conectar taskbar: {:?}", e);
            }
        }
    }
    None
}
