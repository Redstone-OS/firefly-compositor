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
use crate::render::RenderEngine;
use gfx_types::Size;
use redpowder::ipc::{Port, SharedMemory};
use redpowder::syscall::SysResult;
use redpowder::window::{
    opcodes, CommitBufferRequest, CreateWindowRequest, WindowCreatedResponse, COMPOSITOR_PORT,
    MAX_MSG_SIZE,
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

    /// Motor de renderização
    render_engine: RenderEngine,

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
        crate::println!("[Server] Inicializando...");

        // Criar porta nomeada para receber requisições
        let port = Port::create(COMPOSITOR_PORT, 128)?;
        crate::println!("[Server] Porta '{}' criada", COMPOSITOR_PORT);

        // Obter informações do display
        let display_info = redpowder::graphics::get_framebuffer_info()?;
        crate::println!(
            "[Server] Display: {}x{}",
            display_info.width,
            display_info.height
        );

        // Converter para gfx_types::DisplayInfo
        let gfx_display_info = gfx_types::DisplayInfo {
            id: 0,
            width: display_info.width,
            height: display_info.height,
            refresh_rate_mhz: 60000,
            format: gfx_types::PixelFormat::ARGB8888,
            stride: display_info.stride * 4,
        };

        // Inicializar motor de renderização
        let render_engine = RenderEngine::new(gfx_display_info);

        Ok(Self {
            port,
            render_engine,
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
        let mut loop_count = 0u64;

        while self.running {
            loop_count += 1;

            // Log a cada 600 iterações (~10 segundos)
            if loop_count % 600 == 0 {
                let (_, win_count) = self.render_engine.stats();
                crate::println!("[Compositor] Loop ativo, {} janelas", win_count);
            }

            // 1. Processar mensagens IPC (non-blocking)
            self.process_messages(&mut msg_buf)?;

            // 2. Atualizar estado de input
            // Nota: Erros são silenciosamente ignorados
            let _ = self.input.update();

            // 3. Renderizar frame (SEMPRE - não apenas quando há mudanças)
            self.render_engine.render()?;

            // 4. Atualizar estatísticas
            self.update_stats();

            // 5. Estabilizar Framerate
            let _ = redpowder::time::sleep(FRAME_INTERVAL_MS);
        }

        Ok(())
    }

    /// Processa mensagens IPC pendentes.
    /// Processa apenas UMA mensagem por chamada para evitar race conditions.
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

        crate::println!(
            "[Server] CreateWindow: {}x{} em ({}, {})",
            req.width,
            req.height,
            req.x,
            req.y
        );

        // Criar memória compartilhada para o buffer da janela
        let buffer_size = (req.width * req.height * 4) as usize;
        let mut shm = match SharedMemory::create(buffer_size) {
            Ok(s) => s,
            Err(e) => {
                crate::println!("[Server] Falha ao criar SHM: {:?}", e);
                return Ok(());
            }
        };

        // Inicializar memória com PRETO (0xFF000000)
        let pixel_count = (req.width * req.height) as usize;
        let pixels =
            unsafe { core::slice::from_raw_parts_mut(shm.as_mut_ptr() as *mut u32, pixel_count) };
        for pixel in pixels.iter_mut() {
            *pixel = 0xFF000000;
        }
        // Verificar se escrevemos corretamente
        crate::println!("[Server] Primeiro pixel apos init: {:#x}", pixels[0]);

        let shm_id = shm.id();

        // Criar janela no render engine
        let size = Size::new(req.width, req.height);
        let window_id = self.render_engine.create_window(size, shm);

        // Posicionar janela
        self.render_engine
            .move_window(window_id, req.x as i32, req.y as i32);

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
            window_id,
            shm_handle: shm_id.0,
            buffer_size: buffer_size as u64,
        };

        // Enviar resposta
        let resp_bytes = unsafe {
            core::slice::from_raw_parts(
                &response as *const _ as *const u8,
                core::mem::size_of::<WindowCreatedResponse>(),
            )
        };

        let _ = reply_port.send(resp_bytes, 0);
        crate::println!("[Server] Resposta enviada para porta '{}'", port_name);

        crate::println!(
            "[Server] Janela {} criada ({}x{}) SHM: {}",
            window_id,
            req.width,
            req.height,
            shm_id.0
        );

        // Detectar janelas fullscreen e atribuir layer Background
        // Isso garante que o Shell (desktop background) seja renderizado primeiro
        let display_size = self.render_engine.size();
        if req.width == display_size.width
            && req.height == display_size.height
            && req.x == 0
            && req.y == 0
        {
            self.render_engine
                .set_window_layer(window_id, gfx_types::LayerType::Background);
            crate::println!(
                "[Server] Janela {} é fullscreen -> layer Background",
                window_id
            );
        }

        Ok(())
    }

    /// Processa requisição de commit de buffer.
    fn handle_commit_buffer(&mut self, data: &[u8]) -> SysResult<()> {
        let req = unsafe { &*(data.as_ptr() as *const CommitBufferRequest) };
        crate::println!(
            "[Server] COMMIT_BUFFER recebido para janela {}",
            req.window_id
        );
        // Marcar que a janela tem conteúdo (primeira vez que recebe commit)
        self.render_engine.mark_window_has_content(req.window_id);
        // Marcar área como danificada para re-renderização
        self.render_engine.mark_damage(req.window_id);
        Ok(())
    }

    /// Processa requisição de destruição de janela.
    fn handle_destroy_window(&mut self, _data: &[u8]) -> SysResult<()> {
        // TODO: Implementar destruição
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
            let (frames, windows) = self.render_engine.stats();
            crate::println!("[Server] {} frames, {} janelas ativas", frames, windows);
        }
    }
}
