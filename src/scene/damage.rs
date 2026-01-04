//! # Scene - Damage Tracker
//!
//! Sistema de rastreamento de áreas danificadas para otimização de renderização.

use alloc::vec::Vec;
use gfx_types::geometry::Rect;

// =============================================================================
// DAMAGE TRACKER
// =============================================================================

/// Rastreador de áreas danificadas.
///
/// Mantém uma lista de retângulos que precisam ser redesenhados,
/// otimizando a renderização ao evitar redesenhar a tela inteira.
pub struct DamageTracker {
    /// Regiões danificadas.
    regions: Vec<Rect>,
    /// Máximo de regiões antes de colapsar.
    max_regions: usize,
    /// Flag de dano total (tela inteira).
    full_damage: bool,
    /// Bounds da tela.
    screen_rect: Rect,
}

impl DamageTracker {
    /// Cria novo tracker.
    pub fn new() -> Self {
        Self {
            regions: Vec::with_capacity(16),
            max_regions: 16,
            full_damage: true, // Primeiro frame sempre é full
            screen_rect: Rect::ZERO,
        }
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Cria tracker com tamanho de tela.
    pub fn with_size(width: u32, height: u32) -> Self {
        Self {
            regions: Vec::with_capacity(16),
            max_regions: 16,
            full_damage: true,
            screen_rect: Rect::new(0, 0, width, height),
        }
    }

    /// Define tamanho da tela.
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.screen_rect = Rect::new(0, 0, width, height);
    }

    /// Adiciona região danificada.
    pub fn add(&mut self, rect: Rect) {
        if rect.is_empty() {
            return;
        }

        // Clip à tela
        let clipped = match rect.intersection(&self.screen_rect) {
            Some(r) => r,
            None => return,
        };

        // Tentar merge com região existente
        for existing in &mut self.regions {
            if existing.intersects(&clipped) {
                *existing = existing.union(&clipped);
                return;
            }
        }

        self.regions.push(clipped);

        // Colapsar se muitas regiões
        if self.regions.len() > self.max_regions {
            self.collapse();
        }
    }

    /// Marca a tela inteira como danificada.
    pub fn damage_full(&mut self, width: u32, height: u32) {
        self.screen_rect = Rect::new(0, 0, width, height);
        self.full_damage = true;
        self.regions.clear();
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna se há alguma região danificada.
    #[inline]
    pub fn has_damage(&self) -> bool {
        self.full_damage || !self.regions.is_empty()
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna se é dano total.
    #[inline]
    pub fn is_full_damage(&self) -> bool {
        self.full_damage
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna as regiões danificadas.
    pub fn regions(&self) -> &[Rect] {
        &self.regions
    }

    /// Retorna o bounding box de todo o dano.
    pub fn bounding_box(&self) -> Rect {
        if self.full_damage {
            return self.screen_rect;
        }

        if self.regions.is_empty() {
            return Rect::ZERO;
        }

        let mut bounds = self.regions[0];
        for rect in &self.regions[1..] {
            bounds = bounds.union(rect);
        }
        bounds
    }

    /// Limpa todas as regiões.
    pub fn clear(&mut self) {
        self.regions.clear();
        self.full_damage = false;
    }

    /// Colapsa todas as regiões em uma só.
    fn collapse(&mut self) {
        if self.regions.len() <= 1 {
            return;
        }

        let bounds = self.bounding_box();
        self.regions.clear();
        self.regions.push(bounds);
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna e limpa as regiões.
    pub fn take(&mut self) -> Vec<Rect> {
        let mut result = core::mem::take(&mut self.regions);
        if self.full_damage {
            result.clear();
            result.push(self.screen_rect);
        }
        self.full_damage = false;
        result
    }
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self::new()
    }
}
