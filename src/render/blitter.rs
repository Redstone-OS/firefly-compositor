//! # Blitter
//!
//! Operações de cópia de pixels otimizadas.

use gfx_types::{Color, Point, Rect, Size};

/// Blitter - operações de cópia de pixels.
pub struct Blitter;

impl Blitter {
    /// Copia região de src para dst (sem alpha, opaco).
    ///
    /// Copia linha-a-linha para máxima performance.
    #[inline]
    pub fn blit_opaque(
        dst: &mut [u32],
        dst_size: Size,
        src: &[u32],
        src_size: Size,
        src_rect: Rect,
        dst_point: Point,
    ) {
        let src_stride = src_size.width as usize;
        let dst_stride = dst_size.width as usize;

        // Clampar aos limites
        let copy_width = src_rect.width as usize;
        let copy_height = src_rect.height as usize;

        // Debug: primeira chamada apenas
        static mut BLIT_DEBUG: bool = false;
        unsafe {
            if !BLIT_DEBUG {
                BLIT_DEBUG = true;
                crate::println!("[Blit] dst_size: {}x{}", dst_size.width, dst_size.height);
                crate::println!("[Blit] src_size: {}x{}", src_size.width, src_size.height);
                crate::println!("[Blit] dst_point: ({}, {})", dst_point.x, dst_point.y);
                crate::println!("[Blit] copy: {}x{}", copy_width, copy_height);
                crate::println!("[Blit] src.len={}, dst.len={}", src.len(), dst.len());
            }
        }

        let mut pixels_copied = 0usize;

        for y in 0..copy_height {
            let src_y = src_rect.y as usize + y;
            let dst_y = dst_point.y as usize + y;

            if src_y >= src_size.height as usize || dst_y >= dst_size.height as usize {
                continue;
            }

            let src_start = src_y * src_stride + src_rect.x as usize;
            let dst_start = dst_y * dst_stride + dst_point.x as usize;

            let src_end = (src_start + copy_width).min(src.len());
            let dst_end = (dst_start + copy_width).min(dst.len());

            let actual_width = (src_end - src_start).min(dst_end - dst_start);

            if actual_width > 0 && dst_start < dst.len() && src_start < src.len() {
                dst[dst_start..dst_start + actual_width]
                    .copy_from_slice(&src[src_start..src_start + actual_width]);
                pixels_copied += actual_width;
            }
        }

        // Debug quantos pixels foram copiados
        unsafe {
            static mut COPY_DEBUG: bool = false;
            if !COPY_DEBUG {
                COPY_DEBUG = true;
                crate::println!("[Blit] Total pixels copiados: {}", pixels_copied);
            }
        }
    }

    /// Copia com verificação de alpha (para superfícies transparentes).
    #[inline]
    pub fn blit_alpha(
        dst: &mut [u32],
        dst_size: Size,
        src: &[u32],
        src_size: Size,
        src_rect: Rect,
        dst_point: Point,
    ) {
        let src_stride = src_size.width as usize;
        let dst_stride = dst_size.width as usize;

        for y in 0..src_rect.height as usize {
            let src_y = src_rect.y as usize + y;
            let dst_y = dst_point.y as usize + y;

            if src_y >= src_size.height as usize || dst_y >= dst_size.height as usize {
                continue;
            }

            for x in 0..src_rect.width as usize {
                let src_x = src_rect.x as usize + x;
                let dst_x = dst_point.x as usize + x;

                if src_x >= src_size.width as usize || dst_x >= dst_size.width as usize {
                    continue;
                }

                let src_idx = src_y * src_stride + src_x;
                let dst_idx = dst_y * dst_stride + dst_x;

                if src_idx >= src.len() || dst_idx >= dst.len() {
                    continue;
                }

                let src_pixel = src[src_idx];
                let alpha = src_pixel >> 24;

                if alpha == 0xFF {
                    // Totalmente opaco - copia direto
                    dst[dst_idx] = src_pixel;
                } else if alpha > 0 {
                    // Blending necessário
                    dst[dst_idx] = Self::blend(src_pixel, dst[dst_idx], alpha);
                }
                // alpha == 0: transparente, ignora
            }
        }
    }

    /// Preenche retângulo com cor sólida.
    #[inline]
    pub fn fill_rect(dst: &mut [u32], dst_size: Size, rect: Rect, color: Color) {
        let dst_stride = dst_size.width as usize;
        let color_u32 = color.as_u32();

        for y in 0..rect.height as usize {
            let dst_y = rect.y as usize + y;
            if dst_y >= dst_size.height as usize {
                break;
            }

            let start = dst_y * dst_stride + rect.x as usize;
            let end = (start + rect.width as usize).min(dst.len());

            if start < dst.len() {
                dst[start..end].fill(color_u32);
            }
        }
    }

    /// Blending de pixels usando Porter-Duff over.
    #[inline]
    fn blend(src: u32, dst: u32, alpha: u32) -> u32 {
        let inv_alpha = 255 - alpha;

        let sr = (src >> 16) & 0xFF;
        let sg = (src >> 8) & 0xFF;
        let sb = src & 0xFF;

        let dr = (dst >> 16) & 0xFF;
        let dg = (dst >> 8) & 0xFF;
        let db = dst & 0xFF;

        let r = (sr * alpha + dr * inv_alpha) / 255;
        let g = (sg * alpha + dg * inv_alpha) / 255;
        let b = (sb * alpha + db * inv_alpha) / 255;

        0xFF000000 | (r << 16) | (g << 8) | b
    }
}
