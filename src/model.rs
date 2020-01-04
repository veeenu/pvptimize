use crate::error::*;
use crate::gamemaster as gm;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Type {
  Normal,
  Fighting,
  Flying,
  Poison,
  Ground,
  Rock,
  Bug,
  Ghost,
  Steel,
  Fire,
  Water,
  Grass,
  Electric,
  Psychic,
  Ice,
  Dragon,
  Dark,
  Fairy,
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
  energy: u16,
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
        energy: s.energy_delta as u16,
      })
    } else {
       Err(Error::ConversionError(format!(
        "Can't convert gm::ChargedMove into FastMove: {}",
        s.unique_id
      )))
    }
  }
}

impl<'a> FastMove<'a> {
  pub fn stab(&self, p: &Pokemon<'a>) -> bool {
    p.type1 == self.type_ || (p.type2.is_some() && p.type2.unwrap() == self.type_)
  }
}

#[derive(Debug)]
pub struct ChargedMove<'a> {
  uid: &'a str,
  type_: Type,
  power: f64,
  energy: i16,
}

impl<'a> ChargedMove<'a> {
  pub fn stab(&self, p: &Pokemon<'a>) -> bool {
    p.type1 == self.type_ || (p.type2.is_some() && p.type2.unwrap() == self.type_)
  }
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
        energy: s.energy_delta,
      })
    } else {
      Err(Error::ConversionError(format!(
        "Can't convert gm::FastMove into ChargedMove: {}",
        s.unique_id
      )))
    }
  }
}

pub struct Mechanics<'a> {
  gamemaster: &'a gm::GameMaster,

  // pokemon: Vec<Pokemon<'a>>,
  fast_moves: HashMap<&'a str, FastMove<'a>>,
  charged_moves: HashMap<&'a str, ChargedMove<'a>>,

  cp_multiplier: [f64; 79],
  type_effectiveness: HashMap<Type, HashMap<Type, f64>>,
}

impl<'a> Mechanics<'a> {
  pub fn new(gm: &'a gm::GameMaster) -> Result<Mechanics<'a>, Error> {
    let fast_moves = {
      gm.item_templates
        .iter()
        .filter_map(|i| match &i.entry {
          Some(gm::GameMasterEntry::PvPMove(m)) => {
            if m.energy_delta >= 0 {
              Some((m.unique_id.as_str(), FastMove::try_from(m).unwrap()))
            } else {
              None
            }
          },
          _ => None,
        })
        .collect()
    };

    let charged_moves = {
      gm.item_templates
        .iter()
        .filter_map(|i| match &i.entry {
          Some(gm::GameMasterEntry::PvPMove(m)) => {
            if m.energy_delta < 0 {
              Some((m.unique_id.as_str(), ChargedMove::try_from(m).unwrap()))
            } else {
              None
            }
          },
          _ => None,
        })
        .collect()
    };

    Ok(Mechanics {
      gamemaster: gm,
      type_effectiveness: {
        let types = gm
          .item_templates
          .iter()
          .filter_map(|i| match &i.entry {
            Some(gm::GameMasterEntry::TypeEffectiveness(t)) => Some(t),
            _ => None,
          })
          .collect::<Vec<_>>();

        types
          .iter()
          .map(|t| {
            (
              Type::try_from(t.attack_type.as_str()).unwrap(),
              t.effectiveness
                .iter()
                .take(TYPE_ORDERING.len())
                .enumerate()
                .map(|(idx, v)| (TYPE_ORDERING[idx], *v))
                .collect(),
            )
          })
          .collect()
      },
      fast_moves: fast_moves,
      charged_moves: charged_moves,
      cp_multiplier: {
        let pl = gm.item_templates.iter().find(|i| match &i.entry {
          Some(gm::GameMasterEntry::PlayerLevel(_)) => true,
          _ => false,
        });

        let cpm = match pl {
          Some(e) => match &e.entry {
            Some(gm::GameMasterEntry::PlayerLevel(pl)) => Ok(&pl.cp_multiplier),
            _ => Err(Error::ParseError(format!(
              "Couldn't parse unknown structure {:?}",
              pl
            ))),
          },
          None => Err(Error::ParseError(
            "Couldn't find PlayerLevel in GameMaster".to_owned(),
          )),
          /*_ => Err(Error::ParseError(format!(
            "Couldn't parse unknown structure {:?}",
            pl
          ))),*/
        }?;

        if cpm.len() != 40 {
          return Err(Error::ParseError(format!(
            "{} != 40 CPM entries",
            cpm.len()
          )));
        }

        let mut v = [0f64; 79];
        for i in 0..39 {
          v[i * 2] = cpm[i];
          v[i * 2 + 1] = ((cpm[i] * cpm[i] + cpm[i + 1] * cpm[i + 1]) / 2.).sqrt()
        }
        v[78] = cpm[39];

        v
      },
    })
  }

  //
  // TODO
  // Cache this somehow, of course; it is also immutable and derived from the GM
  // like the rest of the Mechanics struct
  //
  pub fn pokemon(&self) -> Result<Vec<Pokemon>, Error> {
    self.gamemaster
      .item_templates
      .iter()
      .filter_map(|i| match &i.entry {
        Some(gm::GameMasterEntry::PokemonSettings(ps)) => {
          let type1 = match Type::try_from(ps.type1.as_str()) {
            Ok(i) => i,
            Err(e) => return Some(Err(e))
          };
          let type2  = match &ps.type2 { 
            Some(s) => match Type::try_from(s.as_str()) {
              Ok(i) => Some(i),
              Err(e) => return Some(Err(e))
            },
            _ => None
          };
          Some(Ok(Pokemon {
            id: &ps.pokemon_id,
            mechanics: &self,
            fast_moves: self.fast_moves
              .iter()
              .filter_map(|(&i, v)| {
                if ps.quick_moves.iter().any(|x| x == i) { 
                  Some(v)
                } else { 
                  None 
                }
              })
              .collect(),
            charged_moves: self.charged_moves
              .iter()
              .filter_map(|(&i, v)| if ps.cinematic_moves.iter().any(|x| x == i) { Some(v) } else { None })
              .collect(),
            type1: type1,
            type2: type2,
            stats: ps.stats,
            type_effectiveness: match type2 {
              Some(t) => self.dual_type_effectiveness(type1, t),
              None => self.type_effectiveness[&type1].clone() // PERFORMANCE I could use Cow but prob it's not really worth it
            }
          }))
        },
        _ => None
      })
      .collect()
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
      .map(|t| {
        (
          *t,
          self.type_effectiveness[t][&a] * self.type_effectiveness[t][&b],
        )
      })
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
  type_effectiveness: HashMap<Type, f64>,
}

pub struct Level {
  pub level: u8,
  pub a_half: bool,
}

impl From<&Level> for u8 {
  fn from(l: &Level) -> u8 {
    (l.level - 1) * 2 + (if l.a_half { 1 } else { 0 })
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
  charged_move2: Option<&'a ChargedMove<'a>>,
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

  pub fn new(pok: &'a Pokemon, level: Level, atk_iv: u8, def_iv: u8, sta_iv: u8, fast_move: &str, charged_move1: &str, charged_move2: Option<&str>) -> Result<PokemonInstance<'a>, Error> {
    Ok(PokemonInstance {
      pokemon: pok,
      atk_iv: atk_iv,
      def_iv: def_iv,
      sta_iv: sta_iv,
      level: level,
      fast_move: match pok.fast_moves.iter().find(|i| i.uid == fast_move) {
        Some(i) => i,
        None => return Err(Error::ParseError(format!("Fast move {} not found for {}", fast_move, pok.id)))
      },
      charged_move1: match pok.charged_moves.iter().find(|i| i.uid == charged_move1) {
        Some(i) => i,
        None => return Err(Error::ParseError(format!("Charged move {} not found for {}", fast_move, pok.id)))
      },
      charged_move2: match charged_move2 {
        Some(cm2) => match pok.charged_moves.iter().find(|&i| i.uid == cm2) {
          Some(&i) => Some(i),
          None => return Err(Error::ParseError(format!("Charged move {} not found for {}", fast_move, pok.id)))
        },
        None => None
      }
    })
  }
}

#[cfg(test)] 
mod tests {

  use super::*;

  #[test]
  fn test_level_conversion() {
    let l: u8 = (&Level { level: 40, a_half: false }).into();
    assert_eq!(l, 78);
    let l: u8 = (&Level { level: 27, a_half: true }).into();
    assert_eq!(l, 53);
  }

  #[test]
  fn test_cp_formula() {
    let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    let gm = serde_json::from_str::<gm::GameMaster>(&gms).unwrap();

    let mech = Mechanics::new(&gm).unwrap();
    let poks = mech.pokemon().unwrap();

    /*println!("{:?}", gm.item_templates.iter().find(|v| {
      if let Some(gm::GameMasterEntry::PvPMove(m)) = &v.entry {
        m.unique_id == "DRAGON_BREATH_FAST"
      } else {
        false
      }
    }));
    println!("{:?}", mech.fast_moves.iter().find(|(&k, _)| k == "DRAGON_BREATH_FAST"));*/
    // Ensure Dragon Breath exists
    assert!(mech.fast_moves.iter().find(|(&k, _)| k == "DRAGON_BREATH_FAST").is_some());

    // Altaria lv28 6/13/14
    let altaria_pok = poks.iter().find(|i| i.id == "ALTARIA").unwrap();
    let altaria = PokemonInstance::new(
      altaria_pok,
      Level { level: 28, a_half: false },
      6, 13, 14,
      "DRAGON_BREATH_FAST",
      "DRAGON_PULSE",
      Some("SKY_ATTACK")
    ).unwrap();

    let noctowl_pok = poks.iter().find(|i| i.id == "NOCTOWL").unwrap();
    let noctowl = PokemonInstance::new(
      noctowl_pok,
      Level { level: 28, a_half: false },
      5, 11, 12,
      "WING_ATTACK_FAST",
      "SKY_ATTACK",
      Some("PSYCHIC")
    ).unwrap();

    let charizard_pok = poks.iter().find(|i| i.id == "CHARIZARD").unwrap();
    let charizard = PokemonInstance::new(
      charizard_pok,
      Level { level: 18, a_half: true },
      11, 8, 15,
      "FIRE_SPIN_FAST",
      "FIRE_BLAST",
      Some("DRAGON_CLAW")
    ).unwrap();

    assert_eq!(altaria.cp(), 1500);
    assert_eq!(noctowl.cp(), 1491);
    assert_eq!(charizard.cp(), 1473);

  }
}