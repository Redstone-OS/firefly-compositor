//! # Scene Module
//!
//! Gerencia a cena gráfica do compositor.
//!
//! ## Componentes
//!
//! - **Window**: Janela de aplicação
//! - **Layer**: Camadas de composição (background, normal, overlay)
//! - **Damage**: Rastreamento de áreas modificadas

pub mod damage;
pub mod layer;
pub mod window;

pub use damage::DamageTracker;
pub use layer::{Layer, LayerManager};
pub use window::{Window, WindowId};
