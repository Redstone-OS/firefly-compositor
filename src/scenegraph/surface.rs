//! # Surface
//!
//! Representa uma janela ou superfície gráfica no compositor.

use redpowder::ipc::{SharedMemory, ShmId};
use redpowder::syscall::SysResult;

pub struct Surface {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub shm: SharedMemory,
    pub z_order: u32,
    pub dirty: bool,
}

impl Surface {
    pub fn new(id: u32, width: u32, height: u32) -> SysResult<Self> {
        // Alocar SHM para a superfície
        // No modelo Wayland, o CLIENTE aloca e passa o FD.
        // No nosso modelo atual (Server-Allocated), o servidor aloca e passa o handle.
        let buffer_size = (width * height * 4) as usize;
        let shm = SharedMemory::create(buffer_size)?;

        Ok(Self {
            id,
            x: 0,
            y: 0,
            width,
            height,
            shm,
            z_order: 0,
            dirty: true,
        })
    }

    pub fn shm_id(&self) -> ShmId {
        self.shm.id()
    }
}
