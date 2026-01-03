//! # UI Module
//!
//! Componentes de interface do compositor.

pub mod cursor;
pub mod decoration;

pub use cursor::{draw as draw_cursor, draw_colored as draw_cursor_colored};
