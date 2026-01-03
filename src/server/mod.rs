//! # Server Module
//!
//! Servidor do compositor Firefly.
//!
//! ## Componentes
//!
//! - **server**: Servidor principal e loop de eventos
//! - **handlers**: Handlers de mensagens IPC
//! - **dispatch**: Dispatch de eventos para clientes
//! - **state**: Estado do servidor (foco, drag, etc)

mod dispatch;
mod handlers;
mod protocol;
mod server;
mod state;

pub use server::Server;
