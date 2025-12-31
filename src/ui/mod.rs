//! # Elementos de Interface do Usuário
//!
//! Módulo contendo elementos visuais gerenciados pelo compositor:
//! - Cursor do mouse
//! - Decorações de janela (bordas, barra de título)
//!
//! ## Nota
//!
//! Estes elementos são desenhados diretamente pelo compositor,
//! não são superfícies de clientes.

pub mod cursor;
pub mod decoration;

// Re-exports serão adicionados conforme necessário
