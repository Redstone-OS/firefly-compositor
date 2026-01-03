//! # Server State
//!
//! Estado do servidor (foco, drag, etc).

/// Estado de arraste de janela.
#[derive(Default)]
pub struct DragState {
    /// Janela sendo arrastada.
    pub window_id: Option<u32>,
    /// Offset X do arraste.
    pub offset_x: i32,
    /// Offset Y do arraste.
    pub offset_y: i32,
}

impl DragState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self, window_id: u32, offset_x: i32, offset_y: i32) {
        self.window_id = Some(window_id);
        self.offset_x = offset_x;
        self.offset_y = offset_y;
    }

    pub fn stop(&mut self) {
        self.window_id = None;
    }

    pub fn is_dragging(&self) -> bool {
        self.window_id.is_some()
    }
}

/// Estado de double-click.
#[derive(Default)]
pub struct ClickState {
    /// Frame do último click.
    pub last_frame: u64,
    /// Janela do último click.
    pub last_window: Option<u32>,
}

impl ClickState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Verifica se é double-click (dentro de 30 frames).
    pub fn is_double_click(&self, window_id: u32, current_frame: u64) -> bool {
        self.last_window == Some(window_id)
            && current_frame > self.last_frame
            && (current_frame - self.last_frame) < 30
    }

    pub fn register(&mut self, window_id: u32, frame: u64) {
        self.last_window = Some(window_id);
        self.last_frame = frame;
    }

    pub fn clear(&mut self) {
        self.last_window = None;
    }
}

/// Estado do mouse.
#[derive(Default)]
pub struct MouseState {
    /// Posição X.
    pub x: i32,
    /// Posição Y.
    pub y: i32,
    /// Botões no frame anterior.
    pub prev_buttons: u32,
}

impl MouseState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn save_buttons(&mut self, buttons: u32) {
        self.prev_buttons = buttons;
    }

    /// Retorna true se botão esquerdo foi pressionado neste frame.
    pub fn left_just_pressed(&self, current_buttons: u32) -> bool {
        let left_now = (current_buttons & 0x01) != 0;
        let left_was = (self.prev_buttons & 0x01) != 0;
        left_now && !left_was
    }

    /// Retorna true se botão esquerdo foi solto neste frame.
    pub fn left_just_released(&self, current_buttons: u32) -> bool {
        let left_now = (current_buttons & 0x01) != 0;
        let left_was = (self.prev_buttons & 0x01) != 0;
        !left_now && left_was
    }

    /// Retorna true se botão esquerdo está pressionado.
    pub fn left_pressed(&self, current_buttons: u32) -> bool {
        (current_buttons & 0x01) != 0
    }
}
