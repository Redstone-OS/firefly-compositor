//! # Scenegraph
//!
//! Módulo responsável pela organização e gerenciamento das superfícies
//! gráficas (janelas) e sua composição no framebuffer.
//!
//! ## Componentes
//!
//! - `Compositor` - Orquestra a renderização de todas as superfícies
//! - `Surface` - Representa uma janela individual

mod compositor;
mod surface;

// Re-exports públicos
pub use compositor::Compositor;
pub use surface::Surface;
