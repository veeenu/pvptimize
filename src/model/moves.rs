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
pub trait Damage<'a> {
  fn calculate(&self, source: &PokemonInstance<'a>, target: &PokemonInstance<'a>) -> i32;
  fn type_(&self) -> &Type;
  fn stab(&self, p: &Pokemon<'a>) -> bool;
}

// ================
// === FastMove ===
// ================

#[derive(Debug)]
pub struct FastMove<'a> {
  pub uid: &'a str,
  pub type_: Type,
  pub power: f64,
  pub turns: i32,
  pub energy: i32,
}

impl<'a> TryFrom<&'a gm::PvPMove> for FastMove<'a> {
  type Error = Error;

  fn try_from(s: &'a gm::PvPMove) -> Result<Self, Self::Error> {
    if s.energy_delta >= 0 {
      Ok(FastMove {
        uid: s.unique_id.as_str(),
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

// Floor(0.5 ∗ Power ∗ Atk / Def ∗ STAB ∗ Effective) + 1
impl<'a> Damage<'a> for FastMove<'a> {
  fn stab(&self, p: &Pokemon<'a>) -> bool {
    p.type1 == self.type_ || (p.type2.is_some() && p.type2.unwrap() == self.type_)
  }

  fn type_(&self) -> &Type {
    &self.type_
  }

  fn calculate(&self, source: &PokemonInstance<'a>, target: &PokemonInstance<'a>) -> i32 {
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

#[derive(Debug)]
pub struct ChargedMove<'a> {
  pub uid: &'a str,
  pub type_: Type,
  pub power: f64,
  pub energy: i16,
}

impl<'a> TryFrom<&'a gm::PvPMove> for ChargedMove<'a> {
  type Error = Error;

  fn try_from(s: &'a gm::PvPMove) -> Result<Self, Self::Error> {
    if s.energy_delta < 0 {
      Ok(ChargedMove {
        uid: s.unique_id.as_str(),
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

impl<'a> Damage<'a> for ChargedMove<'a> {
  fn stab(&self, p: &Pokemon<'a>) -> bool {
    p.type1 == self.type_ || (p.type2.is_some() && p.type2.unwrap() == self.type_)
  }

  fn type_(&self) -> &Type {
    &self.type_
  }

  fn calculate(&self, source: &PokemonInstance<'a>, target: &PokemonInstance<'a>) -> i32 {
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