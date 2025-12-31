//! # Servidor do Compositor
//!
//! Módulo principal que gerencia o loop de renderização e processa
//! mensagens IPC de clientes (aplicações).
//!
//! ## Responsabilidades
//!
//! - Escutar a porta `firefly.compositor` para requisições
//! - Processar mensagens do protocolo Firefly
//! - Coordenar renderização de frames
//! - Gerenciar entrada de mouse/teclado
//!
//! ## Loop Principal
//!
//! ```text
//! while running {
//!     1. Processar mensagens IPC (non-blocking)
//!     2. Atualizar estado de input
//!     3. Renderizar frame
//!     4. Throttle (16ms = ~60 FPS)
//! }
//! ```

use crate::input::InputManager;
use crate::scenegraph::Compositor;
use redpowder::ipc::Port;
use redpowder::syscall::SysResult;
use redpowder::window::{
    opcodes, CommitBufferRequest, CreateWindowRequest, ProtocolMessage, WindowCreatedResponse,
    COMPOSITOR_PORT, MAX_MSG_SIZE,
};

// ============================================================================
// CONSTANTES
// ============================================================================

/// Intervalo entre frames em milissegundos (~60 FPS)
const FRAME_INTERVAL_MS: u64 = 16;

/// Intervalo para log de estatísticas (em frames)
const STATS_LOG_INTERVAL: u64 = 300; // ~5 segundos

// ============================================================================
// SERVIDOR
// ============================================================================

/// Servidor principal do compositor.
///
/// Gerencia o ciclo de vida do compositor, processando requisições
/// de clientes e coordenando a renderização.
pub struct Server {
    /// Porta IPC para receber requisições
    port: Port,

    /// Compositor de cena
    compositor: Compositor,

    /// Gerenciador de entrada
    input: InputManager,

    /// Flag de controle do loop principal
    running: bool,

    /// Contador de frames renderizados
    frame_count: u64,
}

impl Server {
    /// Cria e inicializa o servidor.
    ///
    /// # Retorna
    ///
    /// `Ok(Server)` pronto para executar, ou `Err` em caso de falha.
    pub fn new() -> SysResult<Self> {
        // Criar porta nomeada para receber requisições
        let port = Port::create(COMPOSITOR_PORT, 128)?;

        // Inicializar compositor
        let compositor = Compositor::new()?;

        Ok(Self {
            port,
            compositor,
            input: InputManager::new(),
            running: true,
            frame_count: 0,
        })
    }

    /// Executa o loop principal do compositor.
    ///
    /// Esta função só retorna em caso de erro fatal ou shutdown.
    pub fn run(&mut self) -> SysResult<()> {
        let mut msg_buf = [0u8; MAX_MSG_SIZE];

        while self.running {
            // 1. Processar mensagens IPC (non-blocking)
            self.process_messages(&mut msg_buf)?;

            // 2. Atualizar estado de input
            // Nota: Erros são silenciosamente ignorados
            let _ = self.input.update();

            // 3. Renderizar frame
            self.compositor.render()?;

            // 4. Atualizar estatísticas
            self.update_stats();

            // 5. Throttle para manter ~60 FPS
            let _ = redpowder::time::sleep(FRAME_INTERVAL_MS);
        }

        Ok(())
    }

    /// Processa mensagens IPC pendentes.
    fn process_messages(&mut self, buf: &mut [u8; MAX_MSG_SIZE]) -> SysResult<()> {
        // Tenta receber mensagem (non-blocking com timeout 0)
        if let Ok(size) = self.port.recv(buf, 0) {
            if size > 0 {
                self.handle_message(&buf[..size])?;
            }
        }
        Ok(())
    }

    /// Processa uma mensagem recebida.
    fn handle_message(&mut self, data: &[u8]) -> SysResult<()> {
        // Verificar tamanho mínimo
        if data.len() < 4 {
            return Ok(());
        }

        // Ler opcode do header
        let opcode = unsafe { *(data.as_ptr() as *const u32) };

        match opcode {
            opcodes::CREATE_WINDOW => self.handle_create_window(data),
            opcodes::COMMIT_BUFFER => self.handle_commit_buffer(data),
            opcodes::DESTROY_WINDOW => self.handle_destroy_window(data),
            _ => {
                crate::println!("[Server] Opcode desconhecido: {:#x}", opcode);
                Ok(())
            }
        }
    }

    /// Processa requisição de criação de janela.
    fn handle_create_window(&mut self, data: &[u8]) -> SysResult<()> {
        // Decodificar requisição
        let req = unsafe { &*(data.as_ptr() as *const CreateWindowRequest) };

        // Criar superfície no compositor
        let surface_id = self.compositor.create_surface(req.width, req.height);
        if surface_id == 0 {
            crate::println!("[Server] Falha ao criar superfície");
            return Ok(());
        }

        // Obter handle da memória compartilhada
        let shm_handle = self.compositor.get_surface_shm(surface_id);

        // Extrair nome da porta de resposta
        let name_len = req
            .reply_port
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(req.reply_port.len());

        let port_name = match core::str::from_utf8(&req.reply_port[..name_len]) {
            Ok(name) => name,
            Err(_) => {
                crate::println!("[Server] Nome de porta inválido");
                return Ok(());
            }
        };

        // Conectar à porta de resposta do cliente
        let reply_port = match Port::connect(port_name) {
            Ok(p) => p,
            Err(_) => {
                crate::println!("[Server] Falha ao conectar em '{}'", port_name);
                return Ok(());
            }
        };

        // Montar resposta
        let response = WindowCreatedResponse {
            op: opcodes::WINDOW_CREATED,
            window_id: surface_id,
            shm_handle: shm_handle.0,
            buffer_size: (req.width * req.height * 4) as u64,
        };

        // Enviar resposta
        let resp_bytes = unsafe {
            core::slice::from_raw_parts(
                &response as *const _ as *const u8,
                core::mem::size_of::<WindowCreatedResponse>(),
            )
        };

        let _ = reply_port.send(resp_bytes, 0);

        crate::println!(
            "[Server] Janela {} criada ({}x{}) para '{}'",
            surface_id,
            req.width,
            req.height,
            port_name
        );

        Ok(())
    }

    /// Processa requisição de commit de buffer.
    fn handle_commit_buffer(&mut self, data: &[u8]) -> SysResult<()> {
        let req = unsafe { &*(data.as_ptr() as *const CommitBufferRequest) };
        self.compositor.mark_damage(req.window_id);
        Ok(())
    }

    /// Processa requisição de destruição de janela.
    fn handle_destroy_window(&mut self, _data: &[u8]) -> SysResult<()> {
        // TODO: Implementar destruição de superfície
        crate::println!("[Server] DESTROY_WINDOW não implementado");
        Ok(())
    }

    /// Atualiza estatísticas e logs periódicos.
    fn update_stats(&mut self) {
        self.frame_count += 1;

        if self.frame_count == 1 {
            crate::println!("[Server] Primeiro frame renderizado!");
        }

        if self.frame_count % STATS_LOG_INTERVAL == 0 {
            crate::println!("[Server] {} frames renderizados", self.frame_count);
        }
    }
}
