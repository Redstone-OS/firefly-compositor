//! # Layer Manager
//!
//! Gerencia camadas de composição.

use super::window::WindowId;
use alloc::vec::Vec;
use gfx_types::LayerType;

/// Uma camada de composição.
pub struct Layer {
    /// Tipo da camada.
    pub layer_type: LayerType,
    /// IDs das janelas nesta camada.
    pub windows: Vec<WindowId>,
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

    /// Adiciona janela à camada.
    pub fn add_window(&mut self, id: WindowId) {
        if !self.windows.contains(&id) {
            self.windows.push(id);
        }
    }

    /// Remove janela da camada.
    pub fn remove_window(&mut self, id: WindowId) {
        self.windows.retain(|w| *w != id);
    }
}

/// Gerenciador de camadas.
pub struct LayerManager {
    layers: [Layer; 5],
}

impl LayerManager {
    /// Cria novo gerenciador.
    pub fn new() -> Self {
        Self {
            layers: [
                Layer::new(LayerType::Background),
                Layer::new(LayerType::Normal),
                Layer::new(LayerType::Panel),
                Layer::new(LayerType::Overlay),
                Layer::new(LayerType::Cursor),
            ],
        }
    }

    /// Obtém camada por tipo.
    pub fn get(&self, layer_type: LayerType) -> &Layer {
        &self.layers[layer_type as usize]
    }

    /// Obtém camada mutável por tipo.
    pub fn get_mut(&mut self, layer_type: LayerType) -> &mut Layer {
        &mut self.layers[layer_type as usize]
    }

    /// Adiciona janela a uma camada.
    pub fn add_window_to_layer(&mut self, id: WindowId, layer_type: LayerType) {
        self.get_mut(layer_type).add_window(id);
    }

    /// Remove janela de qualquer camada.
    pub fn remove_window(&mut self, id: WindowId) {
        for layer in &mut self.layers {
            layer.remove_window(id);
        }
    }

    /// Itera camadas de baixo para cima (ordem de desenho).
    pub fn iter_bottom_to_top(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter().filter(|l| l.visible)
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}
