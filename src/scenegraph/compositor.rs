//! # Compositor
//!
//! Gerencia a composição da cena e renderização no Framebuffer físico.

use super::surface::Surface;
use crate::render::Backbuffer;
use alloc::vec::Vec;
use redpowder::graphics::{get_framebuffer_info, Color};
use redpowder::ipc::ShmId;
use redpowder::syscall::{check_error, SysResult};

pub struct Compositor {
    surfaces: Vec<Surface>,
    backbuffer: Backbuffer, // Usando o renderer implementado anteriormente
    next_surface_id: u32,
}

impl Compositor {
    pub fn new() -> SysResult<Self> {
        // Obter informações do hardware framebuffer
        let info = get_framebuffer_info()?;

        // Criar backbuffer com dimensões reais
        let backbuffer = Backbuffer::new(info.width, info.height);

        Ok(Self {
            surfaces: Vec::new(),
            backbuffer,
            next_surface_id: 1,
        })
    }

    pub fn create_surface(&mut self, width: u32, height: u32) -> u32 {
        let id = self.next_surface_id;
        self.next_surface_id += 1;

        if let Ok(surface) = Surface::new(id, width, height) {
            self.surfaces.push(surface);
            return id;
        }

        0 // ID 0 significa erro
    }

    pub fn get_surface_shm(&self, id: u32) -> ShmId {
        if let Some(surface) = self.surfaces.iter().find(|s| s.id == id) {
            surface.shm_id()
        } else {
            ShmId(0)
        }
    }

    pub fn mark_damage(&mut self, id: u32) {
        if let Some(surface) = self.surfaces.iter_mut().find(|s| s.id == id) {
            surface.dirty = true;
        }
    }

    pub fn render(&mut self) -> SysResult<()> {
        // 1. Limpar tela (ou desenhar background)
        self.backbuffer.clear(Color(0xFF222222)); // Dark Gray

        // 2. Desenhar superfícies (ordenadas por Z?)
        for surface in &self.surfaces {
            // Desenhar buffer da superfície no backbuffer
            // Precisamos acessar os pixels brutos do SHM
            let src_buffer = unsafe {
                core::slice::from_raw_parts(
                    surface.shm.as_ptr() as *const u32,
                    (surface.width * surface.height) as usize,
                )
            };

            // Blit simples (chamada estática para evitar borrow checker error)
            Self::blit_surface(&mut self.backbuffer, surface, src_buffer);
        }

        // 3. Desenhar Cursor (se tivermos InputManager)

        // 4. Present (Swap Buffers / Syscall)
        let _ = self.backbuffer.present();

        Ok(())
    }

    fn blit_surface(backbuffer: &mut Backbuffer, surface: &Surface, pixels: &[u32]) {
        for y in 0..surface.height {
            for x in 0..surface.width {
                let idx = (y * surface.width + x) as usize;
                if idx < pixels.len() {
                    let color = pixels[idx];
                    // Ignora pixels transparentes? (0x00xxxxxx)
                    if (color >> 24) != 0 {
                        backbuffer.put_pixel(
                            (surface.x + x as i32) as i32,
                            (surface.y + y as i32) as i32,
                            Color(color),
                        );
                    }
                }
            }
        }
    }
}
