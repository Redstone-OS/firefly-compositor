//! # Damage Tracker
//!
//! Rastreia regiões modificadas para evitar recomposição completa.

use alloc::vec::Vec;
use gfx_types::Rect;

/// Rastreador de damage (áreas modificadas).
pub struct DamageTracker {
    /// Regiões danificadas no frame atual.
    current: Vec<Rect>,
    /// Limite de rects antes de agrupar tudo.
    max_rects: usize,
}

impl DamageTracker {
    /// Cria novo tracker.
    pub fn new() -> Self {
        Self {
            current: Vec::with_capacity(16),
            max_rects: 16,
        }
    }

    /// Adiciona região danificada.
    pub fn add(&mut self, rect: Rect) {
        if rect.is_empty() {
            return;
        }

        // Tentar merge com rect existente se houver overlap
        for existing in &mut self.current {
            if existing.intersects(&rect) {
                *existing = existing.union(&rect);
                return;
            }
        }

        // Adicionar novo rect
        self.current.push(rect);

        // Se exceder limite, agrupar tudo em um bounding box
        if self.current.len() > self.max_rects {
            self.collapse();
        }
    }

    /// Agrupa todos os rects em um bounding box.
    fn collapse(&mut self) {
        if self.current.len() <= 1 {
            return;
        }

        let mut bounds = self.current[0];
        for rect in &self.current[1..] {
            bounds = bounds.union(rect);
        }

        self.current.clear();
        self.current.push(bounds);
    }

    /// Retorna regiões danificadas.
    pub fn get_damage(&self) -> &[Rect] {
        &self.current
    }

    /// Verifica se há damage.
    pub fn has_damage(&self) -> bool {
        !self.current.is_empty()
    }

    /// Limpa damage para próximo frame.
    pub fn clear(&mut self) {
        self.current.clear();
    }

    /// Retorna e limpa damage (take).
    pub fn take(&mut self) -> Vec<Rect> {
        core::mem::take(&mut self.current)
    }

    /// Marca tela inteira como danificada.
    pub fn damage_full(&mut self, width: u32, height: u32) {
        self.current.clear();
        self.current.push(Rect::new(0, 0, width, height));
    }
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self::new()
    }
}
