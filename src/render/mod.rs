//! # Render Module
//!
//! Motor de renderização do compositor.
//!
//! ## Componentes
//!
//! - **Blitter**: Operações de cópia de pixels otimizadas
//! - **RenderEngine**: Motor de composição principal

pub mod blitter;
pub mod compositor;

pub use blitter::Blitter;
pub use compositor::RenderEngine;
