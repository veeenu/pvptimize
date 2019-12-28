use std::fmt::{self, Display};
use std::convert::TryFrom;
use std::collections::HashMap;
use crate::gamemaster as gm;
use crate::error::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Type {
	Normal, Fighting, Flying, Poison,
	Ground, Rock, Bug, Ghost,
	Steel, Fire, Water, Grass,
	Electric, Psychic, Ice, Dragon,
	Dark, Fairy
}

pub static TYPE_ORDERING: &'static [Type] = &[
  Type::Normal,
  Type::Fighting,
  Type::Flying,
  Type::Poison,
  Type::Ground,
  Type::Rock,
  Type::Bug,
  Type::Ghost,
  Type::Steel,
  Type::Fire,
  Type::Water,
  Type::Grass,
  Type::Electric,
  Type::Psychic,
  Type::Ice,
  Type::Dragon,
  Type::Dark,
  Type::Fairy,
];

impl TryFrom<&str> for Type {
	type Error = Error;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		match s {
			"POKEMON_TYPE_NORMAL" => Ok(Type::Normal),
			"POKEMON_TYPE_FIGHTING" => Ok(Type::Fighting),
			"POKEMON_TYPE_FLYING" => Ok(Type::Flying),
			"POKEMON_TYPE_POISON" => Ok(Type::Poison),
			"POKEMON_TYPE_GROUND" => Ok(Type::Ground),
			"POKEMON_TYPE_ROCK" => Ok(Type::Rock),
			"POKEMON_TYPE_BUG" => Ok(Type::Bug),
			"POKEMON_TYPE_GHOST" => Ok(Type::Ghost),
			"POKEMON_TYPE_STEEL" => Ok(Type::Steel),
			"POKEMON_TYPE_FIRE" => Ok(Type::Fire),
			"POKEMON_TYPE_WATER" => Ok(Type::Water),
			"POKEMON_TYPE_GRASS" => Ok(Type::Grass),
			"POKEMON_TYPE_ELECTRIC" => Ok(Type::Electric),
			"POKEMON_TYPE_PSYCHIC" => Ok(Type::Psychic),
			"POKEMON_TYPE_ICE" => Ok(Type::Ice),
			"POKEMON_TYPE_DRAGON" => Ok(Type::Dragon),
			"POKEMON_TYPE_DARK" => Ok(Type::Dark),
      "POKEMON_TYPE_FAIRY" => Ok(Type::Fairy),
      t => Err(Error::ParseError(format!("Can't parse type {}", t))),
		}
	}
}

#[derive(Debug)]
pub struct FastMove<'a> {
  uid: &'a str,
  type_: Type,
  power: f64,
  turns: u16,
  energy: u16
}

impl<'a> TryFrom<&'a gm::PvPMove> for FastMove<'a> {
	type Error = Error;

	fn try_from(s: &'a gm::PvPMove) -> Result<Self, Self::Error> {
    match s {
      gm::PvPMove::FastMove { unique_id, type_, power, duration_turns, energy_delta, .. } => {
        Ok(FastMove {
          uid: unique_id.as_str(),
          type_: Type::try_from(type_.as_str())
            .map_err(|e| 
              Error::ConversionError(format!("Can't convert gm::FastMove into FastMove: {:?}", e)))?,
          power: *power,
          turns: *duration_turns,
          energy: *energy_delta
        })
      },
      gm::PvPMove::ChargedMove { unique_id, .. } => {
        Err(Error::ConversionError(format!("Can't convert gm::ChargedMove into FastMove: {}", unique_id)))
      }
    }
  }
}

#[derive(Debug)]
pub struct ChargedMove<'a> {
  uid: &'a str,
  type_: Type,
  power: f64,
  energy: i16
}

impl<'a> TryFrom<&'a gm::PvPMove> for ChargedMove<'a> {
	type Error = Error;

	fn try_from(s: &'a gm::PvPMove) -> Result<Self, Self::Error> {
    match s {
      gm::PvPMove::ChargedMove { unique_id, type_, power, energy_delta, .. } => {
        Ok(ChargedMove {
          uid: unique_id.as_str(),
          type_: Type::try_from(type_.as_str())
            .map_err(|e| 
              Error::ConversionError(format!("Can't convert gm::ChargedMove into ChargedMove: {:?}", e)))?,
          power: *power,
          energy: *energy_delta
        })
      },
      gm::PvPMove::FastMove { unique_id, .. } => {
        Err(Error::ConversionError(format!("Can't convert gm::FastMove into ChargedMove: {}", unique_id)))
      }
    }
  }
}

pub struct Mechanics<'a> {
	gamemaster: &'a gm::GameMaster,

  type_effectiveness: HashMap<Type, HashMap<Type, f64>>,
  fast_moves: HashMap<&'a str, FastMove<'a>>,
  charged_moves: HashMap<&'a str, ChargedMove<'a>>,
  
  cp_multiplier: [f64; 79],
}

impl<'a> Mechanics<'a> {
  pub fn new(gm: &'a gm::GameMaster) -> Result<Mechanics<'a>, Error> {
    Ok(Mechanics {
      gamemaster: gm,
      type_effectiveness: {
        let types = gm
          .item_templates
          .iter()
          .filter_map(|i| match &i.entry { 
            Some(gm::GameMasterEntry::TypeEffectiveness(t)) => Some(t), 
            _ => None 
          })
          .collect::<Vec<_>>();

        types.iter()
          .map(|t| (
            Type::try_from(t.attack_type.as_str()).unwrap(),
            t.effectiveness.iter()
              .take(TYPE_ORDERING.len())
              .enumerate()
              .map(|(idx, v)| (
                TYPE_ORDERING[idx], *v
              ))
              .collect()
          ))
          .collect()
      },
      fast_moves: {
        gm
          .item_templates
          .iter()
          .filter_map(|i| match &i.entry {
            Some(gm::GameMasterEntry::PvPMove(m)) => match m {
              gm::PvPMove::FastMove { unique_id, .. } => Some((unique_id.as_str(), FastMove::try_from(m).unwrap())),
              _ => None
            }
            _ => None
          })
          .collect()
      },
      charged_moves: {
        gm
          .item_templates
          .iter()
          .filter_map(|i| match &i.entry {
            Some(gm::GameMasterEntry::PvPMove(m)) => match m {
              gm::PvPMove::ChargedMove { unique_id, .. } => Some((unique_id.as_str(), ChargedMove::try_from(m).unwrap())),
              _ => None
            }
            _ => None
          })
          .collect()
      },
      cp_multiplier: {
        let pl = gm
          .item_templates
          .iter()
          .find(|i| match &i.entry {
            Some(gm::GameMasterEntry::PlayerLevel(pl)) => true,
            _ => false
          });

        let cpm = match pl {
          Some(e) => {
            match &e.entry {
              Some(gm::GameMasterEntry::PlayerLevel(pl)) => Ok(&pl.cp_multiplier),
              _ => Err(Error::ParseError(format!("Couldn't parse unknown structure {:?}", pl)))
            }
          },
          None => Err(Error::ParseError("Couldn't find PlayerLevel in GameMaster".to_owned())),
          _ => Err(Error::ParseError(format!("Couldn't parse unknown structure {:?}", pl)))
        }?;

        if cpm.len() != 40 {
          return Err(Error::ParseError(format!("{} != 40 CPM entries", cpm.len())));
        }
        
        let mut v = [0f64; 79];
        for i in 0..39 {
          v[i * 2] = cpm[i];
          v[i * 2 + 1] = ((cpm[i] * cpm[i] + cpm[i + 1] * cpm[i + 1]) / 2.).sqrt()
        }
        v[79] = cpm[39];

        v
      }
    })
  }

  //
  // TODO
  // This really warrants numerical constrained types, which aren't a thing yet.
  // Could use Results but it would add so much more code and it isn't worth it
  // as the checking should be performed upstream -- no Level into u8 should ever
  // be above 79 in a Pokémon, and if it is, then there's some inconsistency.
  // Should perform the check at the input boundary, i.e. JSON deserialization
  // and whatnot and, until we have more powerful instruments to check for it,
  // assume it's never going to be above 79.
  //
  pub fn cp_multiplier(&self, l: &Level) -> f64 {
    let l: u8 = l.into();
    // if l > 79 {
    //  return Err(Error::BoundsError(format!("Sought CPM for level {} > 40", l)));
    //}
    self.cp_multiplier[l as usize]
  }

  pub fn dual_type_effectiveness(&self, a: Type, b: Type) -> HashMap<Type, f64> {
    TYPE_ORDERING
      .iter()
      .map(|t| 
        (*t, self.type_effectiveness[t][&a] * self.type_effectiveness[t][&b])
      )
      .collect::<HashMap<_, _>>()
  }
}

pub struct Pokemon<'a> {
	pub mechanics: &'a Mechanics<'a>,

	id: &'a String,
  stats: gm::Stats,
  type1: Type,
  type2: Option<Type>,
	fast_moves: Vec<&'a FastMove<'a>>,
  charged_moves: Vec<&'a ChargedMove<'a>>,
  type_effectiveness: HashMap<Type, f64>
}

pub struct Level {
  pub level: u8,
  pub a_half: bool
}

impl From<&Level> for u8 {
  fn from(l: &Level) -> u8 {
    (l.level + 1) * 2 + (if l.a_half { 1 } else { 0 })
  }
}

pub struct PokemonInstance<'a> {
  pokemon: &'a Pokemon<'a>,

  atk_iv: u8,
  def_iv: u8,
  sta_iv: u8,

  level: Level,

  fast_move: &'a FastMove<'a>,
  charged_move1: &'a ChargedMove<'a>,
  charged_move2: Option<&'a ChargedMove<'a>>
}

impl<'a> PokemonInstance<'a> {

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
    f64::max(10., (self.pokemon.stats.base_stamina + self.sta_iv) as f64 * self.cpm())
  }

  pub fn cpm(&self) -> f64 {
    self.pokemon.mechanics.cp_multiplier(&self.level)
  }

  pub fn cp(&self) -> u32 {
    let a = (self.pokemon.stats.base_attack + self.atk_iv) as f64;
    let d = (self.pokemon.stats.base_defense + self.def_iv) as f64;
    let s = (self.pokemon.stats.base_stamina + self.sta_iv) as f64;
    let cpm = self.cpm();
    f64::floor(a * d.sqrt() * s.sqrt() * cpm * cpm) as u32
  }
}