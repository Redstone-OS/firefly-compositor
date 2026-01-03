//! # Server Protocol
//!
//! Estruturas de protocolo IPC do servidor.

/// Request de input vindo do serviço de input.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InputUpdateRequest {
    pub op: u32,
    pub event_type: u32,
    pub key_code: u32,
    pub key_pressed: u32,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_buttons: u32,
}

/// Porta de comunicação com um cliente.
pub struct ClientPort {
    pub window_id: u32,
    pub port: redpowder::ipc::Port,
}
