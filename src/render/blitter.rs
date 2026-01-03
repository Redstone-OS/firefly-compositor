//! # Blitter
//!
//! Operações de cópia de pixels otimizadas.
//!
//! ## Funcionalidades
//!
//! - Blit opaco (cópia rápida)
//! - Blit com alpha blending
//! - Preenchimento de retângulos
//! - Gradientes horizontais/verticais
//! - Sombras e efeitos

use gfx_types::color::{BlendMode, Color};
use gfx_types::geometry::{Point, Rect, Size};

// =============================================================================
// BLITTER
// =============================================================================

/// Blitter - operações de cópia de pixels.
pub struct Blitter;

impl Blitter {
    // =========================================================================
    // BLIT OPACO
    // =========================================================================

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
        // Cálculo de clipping
        let dst_rect = Rect::new(dst_point.x, dst_point.y, src_rect.width, src_rect.height);

        let dst_bounds = Rect::new(0, 0, dst_size.width, dst_size.height);
        let clipped = match dst_rect.intersection(&dst_bounds) {
            Some(r) => r,
            None => return,
        };

        let src_stride = src_size.width as usize;
        let dst_stride = dst_size.width as usize;

        let offset_x = (clipped.x - dst_point.x) as usize;
        let offset_y = (clipped.y - dst_point.y) as usize;

        for y in 0..clipped.height as usize {
            let src_y = src_rect.y as usize + offset_y + y;
            let dst_y = clipped.y as usize + y;

            if src_y >= src_size.height as usize {
                continue;
            }

            let src_start = src_y * src_stride + src_rect.x as usize + offset_x;
            let dst_start = dst_y * dst_stride + clipped.x as usize;
            let copy_width = clipped.width as usize;

            let src_end = (src_start + copy_width).min(src.len());
            let dst_end = (dst_start + copy_width).min(dst.len());
            let actual_width = (src_end - src_start).min(dst_end - dst_start);

            if actual_width > 0 && dst_start < dst.len() && src_start < src.len() {
                dst[dst_start..dst_start + actual_width]
                    .copy_from_slice(&src[src_start..src_start + actual_width]);
            }
        }
    }

    // =========================================================================
    // BLIT COM ALPHA
    // =========================================================================

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
                    dst[dst_idx] = src_pixel;
                } else if alpha > 0 {
                    dst[dst_idx] = blend_over(src_pixel, dst[dst_idx]);
                }
            }
        }
    }

    /// Blit com escala simples (nearest neighbor).
    #[inline]
    pub fn blit_scaled(
        dst: &mut [u32],
        dst_size: Size,
        dst_rect: Rect,
        src: &[u32],
        src_size: Size,
        src_rect: Rect,
    ) {
        let src_stride = src_size.width as usize;
        let dst_stride = dst_size.width as usize;

        let scale_x = src_rect.width as f32 / dst_rect.width as f32;
        let scale_y = src_rect.height as f32 / dst_rect.height as f32;

        for dy in 0..dst_rect.height as usize {
            let dst_y = dst_rect.y as usize + dy;
            if dst_y >= dst_size.height as usize {
                continue;
            }

            let src_y = src_rect.y as usize + (dy as f32 * scale_y) as usize;
            if src_y >= src_size.height as usize {
                continue;
            }

            for dx in 0..dst_rect.width as usize {
                let dst_x = dst_rect.x as usize + dx;
                if dst_x >= dst_size.width as usize {
                    continue;
                }

                let src_x = src_rect.x as usize + (dx as f32 * scale_x) as usize;
                if src_x >= src_size.width as usize {
                    continue;
                }

                let src_idx = src_y * src_stride + src_x;
                let dst_idx = dst_y * dst_stride + dst_x;

                if src_idx < src.len() && dst_idx < dst.len() {
                    let pixel = src[src_idx];
                    let alpha = pixel >> 24;

                    if alpha == 0xFF {
                        dst[dst_idx] = pixel;
                    } else if alpha > 0 {
                        dst[dst_idx] = blend_over(pixel, dst[dst_idx]);
                    }
                }
            }
        }
    }

    // =========================================================================
    // PREENCHIMENTO
    // =========================================================================

    /// Preenche retângulo com cor sólida.
    #[inline]
    pub fn fill_rect(dst: &mut [u32], dst_size: Size, rect: Rect, color: Color) {
        let dst_stride = dst_size.width as usize;
        let color_u32 = color.as_u32();

        // Clipping
        let bounds = Rect::new(0, 0, dst_size.width, dst_size.height);
        let clipped = match rect.intersection(&bounds) {
            Some(r) => r,
            None => return,
        };

        for y in 0..clipped.height as usize {
            let dst_y = clipped.y as usize + y;
            let start = dst_y * dst_stride + clipped.x as usize;
            let end = (start + clipped.width as usize).min(dst.len());

            if start < dst.len() {
                dst[start..end].fill(color_u32);
            }
        }
    }

    /// Preenche retângulo com gradiente horizontal.
    #[inline]
    pub fn fill_gradient_h(
        dst: &mut [u32],
        dst_size: Size,
        rect: Rect,
        color_left: Color,
        color_right: Color,
    ) {
        let dst_stride = dst_size.width as usize;
        let bounds = Rect::new(0, 0, dst_size.width, dst_size.height);
        let clipped = match rect.intersection(&bounds) {
            Some(r) => r,
            None => return,
        };

        for y in 0..clipped.height as usize {
            let dst_y = clipped.y as usize + y;

            for x in 0..clipped.width as usize {
                let dst_x = clipped.x as usize + x;
                let idx = dst_y * dst_stride + dst_x;

                if idx < dst.len() {
                    let t = x as f32 / clipped.width as f32;
                    let color = color_left.lerp(&color_right, t);
                    dst[idx] = color.as_u32();
                }
            }
        }
    }

    /// Preenche retângulo com gradiente vertical.
    #[inline]
    pub fn fill_gradient_v(
        dst: &mut [u32],
        dst_size: Size,
        rect: Rect,
        color_top: Color,
        color_bottom: Color,
    ) {
        let dst_stride = dst_size.width as usize;
        let bounds = Rect::new(0, 0, dst_size.width, dst_size.height);
        let clipped = match rect.intersection(&bounds) {
            Some(r) => r,
            None => return,
        };

        for y in 0..clipped.height as usize {
            let dst_y = clipped.y as usize + y;
            let t = y as f32 / clipped.height as f32;
            let color = color_top.lerp(&color_bottom, t).as_u32();

            let start = dst_y * dst_stride + clipped.x as usize;
            let end = (start + clipped.width as usize).min(dst.len());

            if start < dst.len() {
                dst[start..end].fill(color);
            }
        }
    }

    // =========================================================================
    // EFEITOS
    // =========================================================================

    /// Desenha sombra simples (retângulo com alpha).
    #[inline]
    pub fn draw_shadow(
        dst: &mut [u32],
        dst_size: Size,
        rect: Rect,
        offset: Point,
        blur_radius: u32,
        color: Color,
    ) {
        let shadow_rect = rect.offset(offset.x, offset.y).expand(blur_radius as i32);
        let dst_stride = dst_size.width as usize;
        let bounds = Rect::new(0, 0, dst_size.width, dst_size.height);

        let clipped = match shadow_rect.intersection(&bounds) {
            Some(r) => r,
            None => return,
        };

        let shadow_color = color.as_u32();

        for y in 0..clipped.height as usize {
            let dst_y = clipped.y as usize + y;

            for x in 0..clipped.width as usize {
                let dst_x = clipped.x as usize + x;
                let idx = dst_y * dst_stride + dst_x;

                if idx < dst.len() {
                    dst[idx] = blend_over(shadow_color, dst[idx]);
                }
            }
        }
    }

    /// Desenha borda de retângulo.
    #[inline]
    pub fn stroke_rect(dst: &mut [u32], dst_size: Size, rect: Rect, thickness: u32, color: Color) {
        let t = thickness;
        // Top
        Self::fill_rect(
            dst,
            dst_size,
            Rect::new(rect.x, rect.y, rect.width, t),
            color,
        );
        // Bottom
        Self::fill_rect(
            dst,
            dst_size,
            Rect::new(rect.x, rect.bottom() - t as i32, rect.width, t),
            color,
        );
        // Left
        Self::fill_rect(
            dst,
            dst_size,
            Rect::new(rect.x, rect.y + t as i32, t, rect.height - t * 2),
            color,
        );
        // Right
        Self::fill_rect(
            dst,
            dst_size,
            Rect::new(
                rect.right() - t as i32,
                rect.y + t as i32,
                t,
                rect.height - t * 2,
            ),
            color,
        );
    }

    /// Desenha um pixel com verificação de bounds.
    #[inline]
    pub fn put_pixel(dst: &mut [u32], dst_size: Size, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= dst_size.width as i32 || y >= dst_size.height as i32 {
            return;
        }
        let idx = y as usize * dst_size.width as usize + x as usize;
        if idx < dst.len() {
            dst[idx] = color.as_u32();
        }
    }

    /// Desenha um pixel com alpha blending.
    #[inline]
    pub fn put_pixel_blend(dst: &mut [u32], dst_size: Size, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= dst_size.width as i32 || y >= dst_size.height as i32 {
            return;
        }
        let idx = y as usize * dst_size.width as usize + x as usize;
        if idx < dst.len() {
            dst[idx] = blend_over(color.as_u32(), dst[idx]);
        }
    }
}

// =============================================================================
// BLENDING
// =============================================================================

/// Alpha blend (source over) usando Porter-Duff.
#[inline]
fn blend_over(src: u32, dst: u32) -> u32 {
    let sa = (src >> 24) & 0xFF;

    if sa == 0xFF {
        return src;
    }
    if sa == 0 {
        return dst;
    }

    let sr = (src >> 16) & 0xFF;
    let sg = (src >> 8) & 0xFF;
    let sb = src & 0xFF;

    let dr = (dst >> 16) & 0xFF;
    let dg = (dst >> 8) & 0xFF;
    let db = dst & 0xFF;

    let inv_sa = 255 - sa;

    let out_r = (sr * sa + dr * inv_sa) / 255;
    let out_g = (sg * sa + dg * inv_sa) / 255;
    let out_b = (sb * sa + db * inv_sa) / 255;

    0xFF000000 | (out_r << 16) | (out_g << 8) | out_b
}

/// Alpha blend com alpha de destino.
#[inline]
fn blend_over_with_dst_alpha(src: u32, dst: u32) -> u32 {
    let sa = (src >> 24) & 0xFF;
    let da = (dst >> 24) & 0xFF;

    if sa == 0 {
        return dst;
    }
    if sa == 0xFF || da == 0 {
        return src;
    }

    let sr = (src >> 16) & 0xFF;
    let sg = (src >> 8) & 0xFF;
    let sb = src & 0xFF;

    let dr = (dst >> 16) & 0xFF;
    let dg = (dst >> 8) & 0xFF;
    let db = dst & 0xFF;

    let inv_sa = 255 - sa;
    let out_a = sa + (da * inv_sa / 255);

    if out_a == 0 {
        return 0;
    }

    let out_r = (sr * sa + dr * da * inv_sa / 255) / out_a;
    let out_g = (sg * sa + dg * da * inv_sa / 255) / out_a;
    let out_b = (sb * sa + db * da * inv_sa / 255) / out_a;

    (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b
}
