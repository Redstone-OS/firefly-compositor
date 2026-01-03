//! # Scene Module
//!
//! Gerencia a cena gráfica do compositor.
//!
//! ## Componentes
//!
//! - **Window**: Janela de aplicação com estado completo
//! - **Layer**: Camadas de composição (background, normal, panel, overlay)
//! - **Damage**: Rastreamento de áreas modificadas

pub mod damage;
pub mod layer;
pub mod window;

pub use damage::DamageTracker;
pub use layer::{Layer, LayerManager};
pub use window::{Window, WindowId};
