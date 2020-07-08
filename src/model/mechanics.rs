use crate::error::*;
use crate::gamemaster::{self as gm, GameMaster};
use crate::model::{Type, TYPE_ORDERING};
use crate::model::moves::*;
use crate::model::pokemon::*;

use std::collections::HashMap;
use std::convert::TryFrom;

// =================
// === Mechanics ===
// =================

pub struct Mechanics {
  // gamemaster: &'a gm::GameMaster,

  pokemon: Vec<Pokemon>,
  pub fast_moves: HashMap<String, FastMove>,
  pub charged_moves: HashMap<String, ChargedMove>,

  pub cp_multiplier: [f64; 79],
  pub type_effectiveness: HashMap<Type, HashMap<Type, f64>>,
}

lazy_static! {
  static ref MECHANICS: Mechanics = Mechanics::new().unwrap();
}

impl Mechanics {
  pub fn instance() -> &'static Mechanics {
    &MECHANICS
  }

  fn new() -> Result<Mechanics, Error> {
    let gm = GameMaster::instance();
    let fast_moves = {
      gm.item_templates
        .iter()
        .filter_map(|i| match &i.entry {
          Some(gm::GameMasterEntry::PvPMove(m)) => {
            if m.energy_delta >= 0 {
              Some((m.unique_id.to_owned(), FastMove::try_from(m).unwrap()))
            } else {
              None
            }
          }
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
              Some((m.unique_id.to_owned(), ChargedMove::try_from(m).unwrap()))
            } else {
              None
            }
          }
          _ => None,
        })
        .collect()
    };

    let type_effectiveness = {
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
    };

    Ok(Mechanics {
      pokemon: Mechanics::build_pokemons(&gm, &fast_moves, &charged_moves, &type_effectiveness)?,
      fast_moves,
      charged_moves,
      type_effectiveness,
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

        if cpm.len() != 45 {
          return Err(Error::ParseError(format!(
            "{} != 45 CPM entries",
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

  pub fn pokemon(&self, id: &str) -> Option<Pokemon> {
    self
      .pokemon
      .iter()
      .find(|i| i.id == id)
      .map(Pokemon::clone)
  }

  pub fn pokemon_instance(
    &self, 
    pokemon_id: &str,
    level: Level,
    atk_iv: u16,
    def_iv: u16,
    sta_iv: u16,
    fast_move: &str,
    charged_move1: &str,
    charged_move2: Option<&str>
  ) -> Result<PokemonInstance, Error> {
    if let Some(pok) = self.pokemon(pokemon_id) {
      let charged_move2 = charged_move2.unwrap_or(charged_move1);

      let fast_move = match pok.fast_moves.get(fast_move) {
        Some(i) => i.clone(),
        None => {
          return Err(Error::ParseError(format!(
            "Fast move {} not found for {}",
            fast_move, pok.id
          )))
        }
      };

      let charged_move1 = match pok.charged_moves.get(charged_move1) {
        Some(i) => i.clone(),
        None => {
          return Err(Error::ParseError(format!(
            "Charged move {} not found for {}",
            charged_move1, pok.id
          )))
        }
      };

      let charged_move2 = match pok.charged_moves.get(charged_move2) {
        Some(i) => i.clone(),
        None => {
          return Err(Error::ParseError(format!(
            "Charged move {} not found for {}",
            charged_move2, pok.id
          )))
        }
      };

      let cpm = self.cp_multiplier(&level);

      Ok(PokemonInstance::new(
        pok, level, cpm,
        atk_iv, def_iv, sta_iv,
        fast_move,
        charged_move1,
        charged_move2,
      ))
    } else {
      Err(Error::BoundsError(format!("Could not find pokemon {}", pokemon_id)))
    }
  }

  pub fn fast_move(&self, id: &str) -> Option<FastMove> {
    self
      .fast_moves
      .iter()
      .find(|(k, _)| k == &id)
      .map(|(_, v)| FastMove::clone(v))
  }

  pub fn charged_move(&self, id: &str) -> Option<ChargedMove> {
    self
      .charged_moves
      .iter()
      .find(|(k, _)| k == &id)
      .map(|(_, v)| ChargedMove::clone(v))
  }

  //
  // TODO
  // Cache this somehow, of course; it is also immutable and derived from the GM
  // like the rest of the Mechanics struct
  //
  fn build_pokemons(
    gamemaster: &gm::GameMaster, 
    fast_moves: &HashMap<String, FastMove>,
    charged_moves: &HashMap<String, ChargedMove>,
    type_effectiveness: &HashMap<Type, HashMap<Type, f64>>,
  ) -> Result<Vec<Pokemon>, Error> {
    gamemaster
      .item_templates
      .iter()
      .filter_map(|i| match &i.entry {
        Some(gm::GameMasterEntry::PokemonSettings(ps)) => {
          let type1 = match Type::try_from(ps.type1.as_str()) {
            Ok(i) => i,
            Err(e) => return Some(Err(e)),
          };
          let type2 = match &ps.type2 {
            Some(s) => match Type::try_from(s.as_str()) {
              Ok(i) => Some(i),
              Err(e) => return Some(Err(e)),
            },
            _ => None,
          };
          Some(Ok(Pokemon {
            id: ps.pokemon_id.clone(),
            fast_moves: fast_moves
              .iter()
              .filter_map(|(i, v)| {
                if ps.quick_moves.iter().any(|x| x == i) {
                  Some((i.clone(), v.clone()))
                } else {
                  None
                }
              })
              .collect(),
            charged_moves: charged_moves
              .iter()
              .filter_map(|(i, v)| {
                if ps.cinematic_moves.iter().any(|x| x == i) {
                  Some((i.clone(), v.clone()))
                } else {
                  None
                }
              })
              .collect(),
            type1: type1,
            type2: type2,
            stats: ps.stats,
            type_effectiveness: match type2 {
              Some(t) => Mechanics::defender_dual_type_effectiveness_internal(type_effectiveness, type1, t),
              None => Mechanics::defender_type_effectiveness_internal(type_effectiveness, type1)
              //Some(t) => Mechanics::dual_type_effectiveness_internal(type_effectiveness, type1, t),
              // None => type_effectiveness[&type1].clone(),
            },
          }))
        }
        _ => None,
      })
      .collect()
  }

  //
  // TODO
  // This really warrants numerical constrained types, which aren't a thing yet.
  // Could use Results but it would add so much more code and it isn't worth it
  // as the checking should be performed upstream -- no Level into u16 should ever
  // be above 79 in a Pokémon, and if it is, then there's some inconsistency.
  // Should perform the check at the input boundary, i.e. JSON deserialization
  // and whatnot and, until we have more powerful instruments to check for it,
  // assume it's never going to be above 79.
  //
  pub fn cp_multiplier(&self, l: &Level) -> f64 {
    let l: u16 = l.into();
    // if l > 79 {
    //  return Err(Error::BoundsError(format!("Sought CPM for level {} > 40", l)));
    //}
    self.cp_multiplier[l as usize]
  }

  pub fn defender_type_effectiveness(&self, a: Type) -> HashMap<Type, f64> {
    Mechanics::defender_type_effectiveness_internal(&self.type_effectiveness, a)
  }

  fn defender_type_effectiveness_internal(type_effectiveness: &HashMap<Type, HashMap<Type, f64>>, a: Type) -> HashMap<Type, f64> {
    TYPE_ORDERING
      .iter()
      .map(|t| {
        (
          *t,
          type_effectiveness[t][&a]
        )
      })
      .collect::<HashMap<_, _>>()
  }

  pub fn defender_dual_type_effectiveness(&self, a: Type, b: Type) -> HashMap<Type, f64> {
    Mechanics::defender_dual_type_effectiveness_internal(&self.type_effectiveness, a, b)
  }

  fn defender_dual_type_effectiveness_internal(type_effectiveness: &HashMap<Type, HashMap<Type, f64>>, a: Type, b: Type) -> HashMap<Type, f64> {
    TYPE_ORDERING
      .iter()
      .map(|t| {
        (
          *t,
          type_effectiveness[t][&a] * type_effectiveness[t][&b],
        )
      })
      .collect::<HashMap<_, _>>()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_type_effectiveness() {
    // let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    // let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();
    // let mech = Mechanics::try_from(gm).unwrap();
    let mech = Mechanics::instance();

    let regi = mech.pokemon_instance(
      "REGISTEEL",
      Level { level: 22, a_half: true },
      15, 2, 5,
      "LOCK_ON_FAST",
      "FOCUS_BLAST",
      Some("FLASH_CANNON")
    ).unwrap();

    let machamp = mech.pokemon_instance(
      "MACHAMP",
      Level { level: 22, a_half: true },
      15, 2, 5,
      "COUNTER_FAST",
      "DYNAMIC_PUNCH",
      None
    ).unwrap();

    // TODO transpose effectiveness matrix, these all fail
    assert!(machamp.type_effectiveness(&regi.charged_move2) == 1.);
    assert!(regi.type_effectiveness(&regi.fast_move) < 1.);
    assert!(regi.type_effectiveness(&regi.charged_move1) > 1.);
  }
}
