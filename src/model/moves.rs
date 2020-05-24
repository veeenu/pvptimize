use crate::error::*;
use crate::gamemaster as gm;
use crate::model::Type;
use crate::model::pokemon::*;

use std::convert::TryFrom;

// ====================
// === Damage trait ===
// ====================

// Floor(0.5 ∗ Power ∗ Atk / Def ∗ STAB ∗ Effective) + 1
// https://pokemongohub.net/post/questions-and-answers/move-damage-output-actually-calculated/
pub trait Damage {
  fn calculate(&self, source: &PokemonInstance, target: &PokemonInstance) -> i32;
  fn type_(&self) -> &Type;
  fn stab(&self, p: &Pokemon) -> bool;
}

// ================
// === FastMove ===
// ================

#[derive(Debug, Clone)]
pub struct FastMove {
  pub uid: String,
  pub type_: Type,
  pub power: f64,
  pub turns: i32,
  pub energy: i32,
}

impl TryFrom<&gm::PvPMove> for FastMove {
  type Error = Error;

  fn try_from(s: &gm::PvPMove) -> Result<Self, Self::Error> {
    if s.energy_delta >= 0 {
      Ok(FastMove {
        uid: s.unique_id.clone(),
        type_: Type::try_from(s.type_.as_str()).map_err(|e| {
          Error::ConversionError(format!("Can't convert gm::FastMove into FastMove: {:?}", e))
        })?,
        power: s.power,
        turns: s.duration_turns,
        energy: s.energy_delta,
      })
    } else {
      Err(Error::ConversionError(format!(
        "Can't convert gm::ChargedMove into FastMove: {}",
        s.unique_id
      )))
    }
  }
}

impl TryFrom<gm::PvPMove> for FastMove {
  type Error = Error;

  fn try_from(s: gm::PvPMove) -> Result<Self, Self::Error> {
    FastMove::try_from(&s)
  }
}

// Floor(0.5 ∗ Power ∗ Atk / Def ∗ STAB ∗ Effective) + 1
impl Damage for FastMove {
  fn stab(&self, p: &Pokemon) -> bool {
    p.type1 == self.type_ || (p.type2.is_some() && p.type2.unwrap() == self.type_)
  }

  fn type_(&self) -> &Type {
    &self.type_
  }

  fn calculate(&self, source: &PokemonInstance, target: &PokemonInstance) -> i32 {
    let stab = if source.stab(self) { 1.2 } else { 1.0 };
    let effectiveness = target.type_effectiveness(self);
    (
      (
        1.3 *
        0.5 *
        self.power *
        (source.attack() / target.defense()) *
        stab *
        effectiveness
      ).floor() + 1.0
    ).round() as _
  }
}

// ===================
// === ChargedMove ===
// ===================

#[derive(Debug, Clone)]
pub struct ChargedMove {
  pub uid: String,
  pub type_: Type,
  pub power: f64,
  pub energy: i16,
}

impl TryFrom<&gm::PvPMove> for ChargedMove {
  type Error = Error;

  fn try_from(s: &gm::PvPMove) -> Result<Self, Self::Error> {
    if s.energy_delta < 0 {
      Ok(ChargedMove {
        uid: s.unique_id.clone(),
        type_: Type::try_from(s.type_.as_str()).map_err(|e| {
          Error::ConversionError(format!(
            "Can't convert gm::ChargedMove into ChargedMove: {:?}",
            e
          ))
        })?,
        power: s.power,
        energy: s.energy_delta as _,
      })
    } else {
      Err(Error::ConversionError(format!(
        "Can't convert gm::FastMove into ChargedMove: {}",
        s.unique_id
      )))
    }
  }
}

impl TryFrom<gm::PvPMove> for ChargedMove {
  type Error = Error;

  fn try_from(s: gm::PvPMove) -> Result<Self, Self::Error> {
    ChargedMove::try_from(&s)
  }
}


impl Damage for ChargedMove {
  fn stab(&self, p: &Pokemon) -> bool {
    p.type1 == self.type_ || (p.type2.is_some() && p.type2.unwrap() == self.type_)
  }

  fn type_(&self) -> &Type {
    &self.type_
  }

  fn calculate(&self, source: &PokemonInstance, target: &PokemonInstance) -> i32 {
    let stab = if source.stab(self) { 1.2 } else { 1.0 };
    let effectiveness = target.type_effectiveness(self);
    (
      (
        1.3 *
        0.5 *
        self.power *
        (source.attack() / target.defense()) *
        stab *
        effectiveness
      ).floor() + 1.0
    ).round() as _
  }
}
