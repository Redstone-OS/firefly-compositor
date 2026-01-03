//! # Event Dispatch
//!
//! Dispatch de eventos para clientes.

use redpowder::event::{event_type, InputEvent};
use redpowder::ipc::Port;
use redpowder::window::{opcodes, WindowLifecycleEvent};

use super::protocol::ClientPort;

// =============================================================================
// DISPATCH DE EVENTOS
// =============================================================================

/// Envia evento de teclado para uma janela.
pub fn dispatch_key_event(
    client_ports: &[ClientPort],
    window_id: u32,
    key_code: u32,
    pressed: bool,
) {
    let event = InputEvent {
        op: opcodes::EVENT_INPUT,
        event_type: if pressed {
            event_type::KEY_DOWN
        } else {
            event_type::KEY_UP
        },
        param1: key_code,
        param2: 0,
    };

    send_event_to_window(client_ports, window_id, &event);
}

/// Envia evento de mouse para uma janela.
pub fn dispatch_mouse_event(
    client_ports: &[ClientPort],
    window_id: u32,
    rel_x: i32,
    rel_y: i32,
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
        param1: rel_x as u16 as u32,
        param2: ((rel_y as u16 as u32) << 16) | (buttons & 0xFFFF),
    };

    send_event_to_window(client_ports, window_id, &event);
}

/// Envia evento de lifecycle para a taskbar.
pub fn send_lifecycle_event(
    taskbar_port: Option<&Port>,
    event_type: u32,
    window_id: u32,
    title: &str,
) {
    if let Some(port) = taskbar_port {
        let mut title_buf = [0u8; 64];
        let bytes = title.as_bytes();
        let len = bytes.len().min(64);
        title_buf[..len].copy_from_slice(&bytes[..len]);

        let evt = WindowLifecycleEvent {
            op: opcodes::EVENT_WINDOW_LIFECYCLE,
            event_type,
            window_id,
            title: title_buf,
        };

        let evt_bytes = unsafe {
            core::slice::from_raw_parts(
                &evt as *const _ as *const u8,
                core::mem::size_of::<WindowLifecycleEvent>(),
            )
        };
        let _ = port.send(evt_bytes, 0);
    }
}

/// Envia evento para uma janela específica.
fn send_event_to_window(client_ports: &[ClientPort], window_id: u32, event: &InputEvent) {
    let bytes = unsafe {
        core::slice::from_raw_parts(
            event as *const _ as *const u8,
            core::mem::size_of::<InputEvent>(),
        )
    };

    if let Some(client) = client_ports.iter().find(|c| c.window_id == window_id) {
        let _ = client.port.send(bytes, 0);
    }
}
