//! # Scene - Layer Manager
//!
//! Gerenciamento de camadas de composição.

use alloc::vec::Vec;
use gfx_types::window::LayerType;

use super::window::WindowId;

// =============================================================================
// LAYER
// =============================================================================

// TODO: Revisar no futuro
#[allow(unused)]
/// Uma camada de composição.
pub struct Layer {
    /// Tipo da camada.
    pub layer_type: LayerType,
    /// Janelas nesta camada (ordem de baixo para cima).
    windows: Vec<WindowId>,
    /// Camada está visível.
    pub visible: bool,
}

impl Layer {
    /// Cria nova camada.
    pub fn new(layer_type: LayerType) -> Self {
        Self {
            layer_type,
            windows: Vec::new(),
            visible: true,
        }
    }

    /// Adiciona janela ao topo da camada.
    pub fn add_window(&mut self, id: WindowId) {
        if !self.windows.contains(&id) {
            self.windows.push(id);
        }
    }

    /// Remove janela da camada.
    pub fn remove_window(&mut self, id: WindowId) {
        self.windows.retain(|w| *w != id);
    }

    /// Move janela para o topo.
    pub fn bring_to_front(&mut self, id: WindowId) {
        if let Some(pos) = self.windows.iter().position(|w| *w == id) {
            self.windows.remove(pos);
            self.windows.push(id);
        }
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Move janela para o fundo.
    pub fn send_to_back(&mut self, id: WindowId) {
        if let Some(pos) = self.windows.iter().position(|w| *w == id) {
            self.windows.remove(pos);
            self.windows.insert(0, id);
        }
    }

    /// Retorna janelas de baixo para cima.
    pub fn iter_bottom_to_top(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.windows.iter().copied()
    }

    /// Retorna janelas de cima para baixo.
    pub fn iter_top_to_bottom(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.windows.iter().rev().copied()
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Número de janelas.
    #[inline]
    pub fn len(&self) -> usize {
        self.windows.len()
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna se está vazia.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Contém janela?
    #[inline]
    pub fn contains(&self, id: WindowId) -> bool {
        self.windows.contains(&id)
    }
}

// =============================================================================
// LAYER MANAGER
// =============================================================================

/// Gerenciador de camadas.
pub struct LayerManager {
    /// Camada de background (wallpaper, desktop icons).
    background: Layer,
    /// Camada normal (janelas de aplicações).
    normal: Layer,
    /// Camada top (always on top).
    top: Layer,
    /// Camada de panel (taskbar, dock).
    panel: Layer,
    /// Camada de overlay (menus, popups, notificações).
    overlay: Layer,
    /// Camada de lock (lock screen).
    lock: Layer,
    /// Camada de cursor (sempre no topo).
    cursor: Layer,
}

impl LayerManager {
    /// Cria novo gerenciador.
    pub fn new() -> Self {
        Self {
            background: Layer::new(LayerType::Background),
            normal: Layer::new(LayerType::Normal),
            top: Layer::new(LayerType::Top),
            panel: Layer::new(LayerType::Panel),
            overlay: Layer::new(LayerType::Overlay),
            lock: Layer::new(LayerType::Lock),
            cursor: Layer::new(LayerType::Cursor),
        }
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna referência à camada.
    pub fn get(&self, layer_type: LayerType) -> &Layer {
        match layer_type {
            LayerType::Background => &self.background,
            LayerType::Normal => &self.normal,
            LayerType::Top => &self.top,
            LayerType::Panel => &self.panel,
            LayerType::Overlay => &self.overlay,
            LayerType::Lock => &self.lock,
            LayerType::Cursor => &self.cursor,
        }
    }

    /// Retorna referência mutável à camada.
    pub fn get_mut(&mut self, layer_type: LayerType) -> &mut Layer {
        match layer_type {
            LayerType::Background => &mut self.background,
            LayerType::Normal => &mut self.normal,
            LayerType::Top => &mut self.top,
            LayerType::Panel => &mut self.panel,
            LayerType::Overlay => &mut self.overlay,
            LayerType::Lock => &mut self.lock,
            LayerType::Cursor => &mut self.cursor,
        }
    }

    /// Adiciona janela a uma camada.
    pub fn add_window_to_layer(&mut self, id: WindowId, layer_type: LayerType) {
        self.get_mut(layer_type).add_window(id);
    }

    /// Remove janela de qualquer camada.
    pub fn remove_window(&mut self, id: WindowId) {
        self.background.remove_window(id);
        self.normal.remove_window(id);
        self.top.remove_window(id);
        self.panel.remove_window(id);
        self.overlay.remove_window(id);
        self.lock.remove_window(id);
        self.cursor.remove_window(id);
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Move janela entre camadas.
    pub fn move_window(&mut self, id: WindowId, from: LayerType, to: LayerType) {
        self.get_mut(from).remove_window(id);
        self.get_mut(to).add_window(id);
    }

    /// Itera sobre todas as janelas de baixo para cima (ordem de renderização).
    pub fn iter_bottom_to_top(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.background
            .iter_bottom_to_top()
            .chain(self.normal.iter_bottom_to_top())
            .chain(self.top.iter_bottom_to_top())
            .chain(self.panel.iter_bottom_to_top())
            .chain(self.overlay.iter_bottom_to_top())
            .chain(self.lock.iter_bottom_to_top())
            .chain(self.cursor.iter_bottom_to_top())
    }

    /// Itera sobre todas as janelas de cima para baixo (ordem de hit-testing).
    pub fn iter_top_to_bottom(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.cursor
            .iter_top_to_bottom()
            .chain(self.lock.iter_top_to_bottom())
            .chain(self.overlay.iter_top_to_bottom())
            .chain(self.panel.iter_top_to_bottom())
            .chain(self.top.iter_top_to_bottom())
            .chain(self.normal.iter_top_to_bottom())
            .chain(self.background.iter_top_to_bottom())
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Total de janelas em todas as camadas.
    pub fn total_windows(&self) -> usize {
        self.background.len()
            + self.normal.len()
            + self.top.len()
            + self.panel.len()
            + self.overlay.len()
            + self.lock.len()
            + self.cursor.len()
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}
