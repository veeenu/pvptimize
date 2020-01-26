use std::collections::HashMap;

use crate::error::*;
use crate::gamemaster as gm;
use crate::model::Type;
use crate::model::mechanics::*;
use crate::model::moves::*;

// ===============
// === Pokemon ===
// ===============

pub struct Pokemon<'a> {
  pub mechanics: &'a Mechanics<'a>,

  pub id: &'a String,
  pub stats: gm::Stats,
  pub type1: Type,
  pub type2: Option<Type>,
  pub fast_moves: Vec<&'a FastMove<'a>>,
  pub charged_moves: Vec<&'a ChargedMove<'a>>,
  pub type_effectiveness: HashMap<Type, f64>,
}

// === Level ===

pub struct Level {
  pub level: u8,
  pub a_half: bool,
}

impl From<&Level> for u8 {
  fn from(l: &Level) -> u8 {
    (l.level - 1) * 2 + (if l.a_half { 1 } else { 0 })
  }
}

// =======================
// === PokemonInstance ===
// =======================

pub struct PokemonInstance<'a> {
  pokemon: &'a Pokemon<'a>,

  atk_iv: u8,
  def_iv: u8,
  sta_iv: u8,

  level: Level,

  pub fast_move: &'a FastMove<'a>,
  pub charged_move1: &'a ChargedMove<'a>,
  pub charged_move2: Option<&'a ChargedMove<'a>>,
}

impl<'a> PokemonInstance<'a>
{
  //
  // TODO
  // Would make sense to cache all of the following instead of recomputing,
  // but then again, it's just an extra multiplication -- would be
  // premature optimization
  //
  pub fn attack(&self) -> f64 {
    (self.pokemon.stats.base_attack + self.atk_iv) as f64 * self.cpm()
  }

  pub fn defense(&self) -> f64 {
    (self.pokemon.stats.base_defense + self.def_iv) as f64 * self.cpm()
  }

  pub fn stamina(&self) -> f64 {
    f64::max(
      10.,
      (self.pokemon.stats.base_stamina + self.sta_iv) as f64 * self.cpm(),
    )
  }

  pub fn cpm(&self) -> f64 {
    self.pokemon.mechanics.cp_multiplier(&self.level)
  }

  pub fn cp(&self) -> u32 {
    let a = (self.pokemon.stats.base_attack + self.atk_iv) as f64;
    let d = (self.pokemon.stats.base_defense + self.def_iv) as f64;
    let s = (self.pokemon.stats.base_stamina + self.sta_iv) as f64;
    let cpm = self.cpm();
    f64::floor(a * d.sqrt() * s.sqrt() * cpm * cpm / 10.) as u32
  }

  pub fn stab<M: Damage<'a>>(&self, move_: &M) -> bool {
    move_.stab(&self.pokemon)
  }

  pub fn type_effectiveness<M: Damage<'a>>(&self, move_: &M) -> f64 {
    *self.pokemon.type_effectiveness.get(move_.type_()).unwrap() // UNWRAP SAFE: key should always be defined
  }

  pub fn new(
    pok: &'a Pokemon,
    level: Level,
    atk_iv: u8,
    def_iv: u8,
    sta_iv: u8,
    fast_move: &str,
    charged_move1: &str,
    charged_move2: Option<&str>,
  ) -> Result<PokemonInstance<'a>, Error> {
    Ok(PokemonInstance {
      pokemon: pok,
      atk_iv: atk_iv,
      def_iv: def_iv,
      sta_iv: sta_iv,
      level: level,
      fast_move: match pok.fast_moves.iter().find(|i| i.uid == fast_move) {
        Some(i) => i,
        None => {
          return Err(Error::ParseError(format!(
            "Fast move {} not found for {}",
            fast_move, pok.id
          )))
        }
      },
      charged_move1: match pok.charged_moves.iter().find(|i| i.uid == charged_move1) {
        Some(i) => i,
        None => {
          return Err(Error::ParseError(format!(
            "Charged move {} not found for {}",
            fast_move, pok.id
          )))
        }
      },
      charged_move2: match charged_move2 {
        Some(cm2) => match pok.charged_moves.iter().find(|&i| i.uid == cm2) {
          Some(&i) => Some(i),
          None => {
            return Err(Error::ParseError(format!(
              "Charged move {} not found for {}",
              fast_move, pok.id
            )))
          }
        },
        None => None,
      },
    })
  }
}
