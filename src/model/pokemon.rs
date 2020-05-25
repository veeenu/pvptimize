use std::collections::HashMap;

use crate::error::*;
use crate::gamemaster as gm;
use crate::model::Type;
use crate::model::mechanics::*;
use crate::model::moves::*;

// ===============
// === Pokemon ===
// ===============

#[derive(Debug, Clone)]
pub struct Pokemon {
  pub id: String,
  pub stats: gm::Stats,
  pub type1: Type,
  pub type2: Option<Type>,
  pub fast_moves: Vec<FastMove>,
  pub charged_moves: Vec<ChargedMove>,
  pub type_effectiveness: HashMap<Type, f64>,
}

// === Level ===

pub struct Level {
  pub level: u16,
  pub a_half: bool,
}

impl From<&Level> for u16 {
  fn from(l: &Level) -> u16 {
    (l.level - 1) * 2 + (if l.a_half { 1 } else { 0 })
  }
}

// =======================
// === PokemonInstance ===
// =======================

pub struct PokemonInstance {
  pokemon: Pokemon,

  atk_iv: u16,
  def_iv: u16,
  sta_iv: u16,

  level: Level,
  cpm: f64,

  pub fast_move: FastMove,
  pub charged_move1: ChargedMove,
  pub charged_move2: ChargedMove,
}

impl PokemonInstance {
  //
  // TODO
  // Would make sense to cache all of the following instead of recomputing,
  // but then again, it's just an extra multiplication -- would be
  // premature optimization
  //
  pub fn attack(&self) -> f64 {
    (self.pokemon.stats.base_attack + self.atk_iv) as f64 * self.cpm
  }

  pub fn defense(&self) -> f64 {
    (self.pokemon.stats.base_defense + self.def_iv) as f64 * self.cpm
  }

  pub fn stamina(&self) -> f64 {
    f64::max(
      10.,
      (self.pokemon.stats.base_stamina + self.sta_iv) as f64 * self.cpm,
    )
  }

  pub fn cp(&self) -> u32 {
    let a = (self.pokemon.stats.base_attack + self.atk_iv) as f64;
    let d = (self.pokemon.stats.base_defense + self.def_iv) as f64;
    let s = (self.pokemon.stats.base_stamina + self.sta_iv) as f64;
    let cpm = self.cpm;
    f64::floor(a * d.sqrt() * s.sqrt() * cpm * cpm / 10.) as u32
  }

  pub fn stab<M: Damage>(&self, move_: &M) -> bool {
    move_.stab(&self.pokemon)
  }

  pub fn type_effectiveness<M: Damage>(&self, move_: &M) -> f64 {
    *self.pokemon.type_effectiveness.get(move_.type_()).unwrap() // UNWRAP SAFE: key should always be defined
  }

  pub fn new(
    pokemon: Pokemon,
    level: Level,
    cpm: f64,
    atk_iv: u16,
    def_iv: u16,
    sta_iv: u16,
    fast_move: FastMove,
    charged_move1: ChargedMove,
    charged_move2: ChargedMove,
  ) -> PokemonInstance {
    PokemonInstance {
      pokemon,
      level,
      cpm,
      atk_iv,
      def_iv,
      sta_iv,
      fast_move,
      charged_move1,
      charged_move2,
    }
  }
}
