//! # Módulo de Renderização
//!
//! Este módulo implementa o buffer de vídeo em RAM (backbuffer) e a lógica
//! de apresentação no framebuffer físico.
//!
//! ## Arquitetura
//!
//! O compositor desenha todas as superfícies em um backbuffer na RAM,
//! evitando syscalls excessivas. Quando o frame está pronto, o backbuffer
//! é copiado para o framebuffer do kernel via syscall `FB_WRITE`.
//!
//! ## Problema de Stride
//!
//! O framebuffer físico pode ter um stride (bytes por linha) maior que
//! width * 4. Isso ocorre por razões de alinhamento de hardware.
//! Portanto, NÃO podemos simplesmente copiar o buffer como um bloco
//! contíguo - precisamos copiar linha por linha.

use alloc::vec;
use alloc::vec::Vec;
use redpowder::graphics::{get_framebuffer_info, write_framebuffer, Color, FramebufferInfo};
use redpowder::syscall::SysResult;

// ============================================================================
// BACKBUFFER
// ============================================================================

/// Buffer de pixels em RAM para composição.
///
/// O backbuffer mantém uma cópia local de toda a tela, permitindo
/// operações de desenho rápidas sem syscalls. Apenas no `present()`
/// é que os dados são enviados ao kernel.
pub struct Backbuffer {
    /// Largura em pixels
    pub width: u32,
    /// Altura em pixels
    pub height: u32,
    /// Stride do framebuffer físico em bytes (pode ser > width * 4)
    pub stride: u32,
    /// Buffer de pixels ARGB (formato 0xAARRGGBB)
    pub buffer: Vec<u32>,
}

impl Backbuffer {
    /// Cria um novo backbuffer com as dimensões do framebuffer físico.
    ///
    /// # Retorna
    ///
    /// `Ok(Backbuffer)` com as dimensões corretas, ou `Err` se não
    /// conseguir obter informações do framebuffer.
    pub fn new() -> SysResult<Self> {
        let info = get_framebuffer_info()?;
        let size = (info.width * info.height) as usize;

        // Inicializa com preto opaco
        let buffer = vec![0xFF000000u32; size];

        Ok(Self {
            width: info.width,
            height: info.height,
            stride: info.stride,
            buffer,
        })
    }

    /// Cria backbuffer com dimensões específicas (para testes).
    pub fn with_dimensions(width: u32, height: u32, stride: u32) -> Self {
        let size = (width * height) as usize;
        let buffer = vec![0xFF000000u32; size];

        Self {
            width,
            height,
            stride,
            buffer,
        }
    }

    /// Desenha um pixel no buffer.
    ///
    /// # Parâmetros
    ///
    /// * `x` - Coordenada X (pixels)
    /// * `y` - Coordenada Y (pixels)
    /// * `color` - Cor no formato ARGB
    ///
    /// Pixels fora dos limites são silenciosamente ignorados.
    #[inline]
    pub fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        // Verificação de bounds
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let offset = (y as usize * self.width as usize) + x as usize;
        if offset < self.buffer.len() {
            self.buffer[offset] = color.0;
        }
    }

    /// Preenche um retângulo com uma cor sólida.
    ///
    /// # Parâmetros
    ///
    /// * `x`, `y` - Canto superior esquerdo
    /// * `w`, `h` - Largura e altura
    /// * `color` - Cor de preenchimento
    ///
    /// Áreas fora dos limites são automaticamente recortadas (clipping).
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        // Clipping
        let x1 = x.max(0) as u32;
        let y1 = y.max(0) as u32;
        let x2 = ((x + w as i32) as u32).min(self.width);
        let y2 = ((y + h as i32) as u32).min(self.height);

        if x1 >= x2 || y1 >= y2 {
            return;
        }

        let rect_w = (x2 - x1) as usize;
        let color_u32 = color.0;

        // Preenchimento linha por linha (otimizado)
        for cur_y in y1..y2 {
            let start = (cur_y as usize * self.width as usize) + x1 as usize;
            let end = start + rect_w;

            if end <= self.buffer.len() {
                self.buffer[start..end].fill(color_u32);
            }
        }
    }

    /// Limpa todo o buffer com uma cor sólida.
    ///
    /// # Parâmetros
    ///
    /// * `color` - Cor para preencher toda a tela
    #[inline]
    pub fn clear(&mut self, color: Color) {
        self.buffer.fill(color.0);
    }

    /// Envia o backbuffer para o framebuffer físico via syscall.
    ///
    /// # Nota
    ///
    /// Como o stride do framebuffer é igual a width*4 (sem padding),
    /// podemos enviar o buffer inteiro de uma vez.
    ///
    /// # Retorna
    ///
    /// `true` se a apresentação foi bem-sucedida, `false` caso contrário.
    pub fn present(&self) -> bool {
        // Converter buffer de u32 para bytes
        let byte_slice = unsafe {
            core::slice::from_raw_parts(self.buffer.as_ptr() as *const u8, self.buffer.len() * 4)
        };

        // Enviar todo o buffer de uma vez
        match write_framebuffer(0, byte_slice) {
            Ok(_) => true,
            Err(_) => {
                crate::println!("[Backbuffer] ERRO ao escrever framebuffer!");
                false
            }
        }
    }
}

// ============================================================================
// TESTES
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_pixel_bounds() {
        let mut bb = Backbuffer::with_dimensions(100, 100, 400);

        // Pixels válidos
        bb.put_pixel(0, 0, Color(0xFFFFFFFF));
        bb.put_pixel(99, 99, Color(0xFFFFFFFF));

        // Pixels fora dos limites (não devem causar panic)
        bb.put_pixel(-1, 0, Color(0xFFFFFFFF));
        bb.put_pixel(100, 0, Color(0xFFFFFFFF));
        bb.put_pixel(0, -1, Color(0xFFFFFFFF));
        bb.put_pixel(0, 100, Color(0xFFFFFFFF));
    }

    #[test]
    fn test_clear() {
        let mut bb = Backbuffer::with_dimensions(10, 10, 40);
        bb.clear(Color(0xFF222222));

        assert!(bb.buffer.iter().all(|&p| p == 0xFF222222));
    }
}
