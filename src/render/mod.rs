//! # Render Module
//!
//! Motor de renderização do compositor.
//!
//! ## Responsabilidades
//!
//! - Composição de janelas no backbuffer
//! - Operações de blit otimizadas
//! - Gerenciamento do backbuffer

pub mod blitter;
pub mod compositor;

pub use blitter::Blitter;
pub use compositor::RenderEngine;
