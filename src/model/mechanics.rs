use crate::error::*;
use crate::gamemaster as gm;
use crate::model::{Type, TYPE_ORDERING};
use crate::model::moves::*;
use crate::model::pokemon::*;

use std::collections::HashMap;
use std::convert::TryFrom;

// =================
// === Mechanics ===
// =================

pub struct Mechanics<'a> {
  gamemaster: &'a gm::GameMaster,

  // pokemon: Vec<Pokemon<'a>>,
  pub fast_moves: HashMap<&'a str, FastMove<'a>>,
  pub charged_moves: HashMap<&'a str, ChargedMove<'a>>,

  pub cp_multiplier: [f64; 79],
  pub type_effectiveness: HashMap<Type, HashMap<Type, f64>>,
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
              Some((m.unique_id.as_str(), ChargedMove::try_from(m).unwrap()))
            } else {
              None
            }
          }
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

  //
  // TODO
  // Cache this somehow, of course; it is also immutable and derived from the GM
  // like the rest of the Mechanics struct
  //
  pub fn pokemon(&self) -> Result<Vec<Pokemon>, Error> {
    self
      .gamemaster
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
            id: &ps.pokemon_id,
            mechanics: &self,
            fast_moves: self
              .fast_moves
              .iter()
              .filter_map(|(&i, v)| {
                if ps.quick_moves.iter().any(|x| x == i) {
                  Some(v)
                } else {
                  None
                }
              })
              .collect(),
            charged_moves: self
              .charged_moves
              .iter()
              .filter_map(|(&i, v)| {
                if ps.cinematic_moves.iter().any(|x| x == i) {
                  Some(v)
                } else {
                  None
                }
              })
              .collect(),
            type1: type1,
            type2: type2,
            stats: ps.stats,
            type_effectiveness: match type2 {
              Some(t) => self.dual_type_effectiveness(type1, t),
              None => self.type_effectiveness[&type1].clone(), // PERFORMANCE I could use Cow but prob it's not really worth it
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