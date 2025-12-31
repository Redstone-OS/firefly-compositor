//! # Server Core
//!
//! Gerencia conexões IPC e despacha requisições.

use crate::input::InputManager;
use crate::scenegraph::Compositor;
use redpowder::ipc::Port;
use redpowder::syscall::SysResult;
use redpowder::window::{opcodes, ErrorResponse, ProtocolMessage, WindowCreatedResponse};

pub struct Server {
    port: Port,
    compositor: Compositor,
    input: InputManager,
    running: bool,
}

impl Server {
    /// Inicializa o servidor
    pub fn new() -> SysResult<Self> {
        // Criar porta nomeada para o compositor
        let port = Port::create(redpowder::window::COMPOSITOR_PORT, 128)?;

        Ok(Self {
            port,
            compositor: Compositor::new()?,
            input: InputManager::new(),
            running: true,
        })
    }

    /// Loop principal
    pub fn run(&mut self) -> SysResult<()> {
        let mut msg_buf = [0u8; redpowder::window::MAX_MSG_SIZE];

        while self.running {
            // 1. Processar mensagens IPC (Non-blocking)
            // msg_buf é usado para ler.
            // Port::recv com timeout 0 é non-blocking (ou quase).
            if let Ok(size) = self.port.recv(&mut msg_buf, 0) {
                if size > 0 {
                    self.handle_message(&msg_buf[..size])?;
                }
            }

            // 2. Processar Input (Kernel)
            self.input.update()?;

            // 3. Renderizar (Compositor)
            // Futuro: Passar input para desenhar cursor
            self.compositor.render()?;

            // Throttle (aprox 60 FPS)
            redpowder::time::sleep(16)?;
        }

        Ok(())
    }

    fn handle_message(&mut self, data: &[u8]) -> SysResult<()> {
        if data.len() < 4 {
            return Ok(());
        }

        let header = unsafe { *(data.as_ptr() as *const u32) };
        // Cast unsafe para ProtocolMessage
        let msg = unsafe { &*(data.as_ptr() as *const ProtocolMessage) };

        match header {
            opcodes::CREATE_WINDOW => {
                let req = unsafe { msg.create_req };
                // Criar superfície
                let surface_id = self.compositor.create_surface(req.width, req.height);
                // Obter handle SHM
                let shm_handle = self.compositor.get_surface_shm(surface_id);

                // Responder
                let _resp = WindowCreatedResponse {
                    op: opcodes::WINDOW_CREATED,
                    window_id: surface_id,
                    shm_handle: shm_handle.0, // Unwrap wrapper
                    buffer_size: (req.width * req.height * 4) as u64,
                };

                // TODO: Enviar resposta para a PORTA DO CLIENTE.
                // Atualmente, Port::recv não nos dá o remetente.
                // Precisamos de um mecanismo de "sessão" ou "client_fd".

                crate::println!("Window created: {}", surface_id);
            }
            opcodes::COMMIT_BUFFER => {
                let req = unsafe { msg.buf_req };
                self.compositor.mark_damage(req.window_id);
            }
            _ => {}
        }

        Ok(())
    }
}
