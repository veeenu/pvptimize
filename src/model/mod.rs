use std::collections::HashMap;
use std::convert::TryFrom;

mod battle;
mod mechanics;
mod moves;
mod pokemon;

use crate::error::*;
use crate::gamemaster as gm;
use battle::*;
use mechanics::*;
use moves::*;
use pokemon::*;

pub use mechanics::Mechanics;

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
#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_level_conversion() {
    let l: u8 = (&Level {
      level: 40,
      a_half: false,
    })
      .into();
    assert_eq!(l, 78);
    let l: u8 = (&Level {
      level: 27,
      a_half: true,
    })
      .into();
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
    assert!(mech
      .fast_moves
      .iter()
      .find(|(&k, _)| k == "DRAGON_BREATH_FAST")
      .is_some());

    // Altaria lv28 6/13/14
    let altaria_pok = poks.iter().find(|i| i.id == "ALTARIA").unwrap();
    let altaria = PokemonInstance::new(
      altaria_pok,
      Level {
        level: 28,
        a_half: false,
      },
      6,
      13,
      14,
      "DRAGON_BREATH_FAST",
      "DRAGON_PULSE",
      Some("SKY_ATTACK"),
    )
    .unwrap();

    let noctowl_pok = poks.iter().find(|i| i.id == "NOCTOWL").unwrap();
    let noctowl = PokemonInstance::new(
      noctowl_pok,
      Level {
        level: 28,
        a_half: false,
      },
      5,
      11,
      12,
      "WING_ATTACK_FAST",
      "SKY_ATTACK",
      Some("PSYCHIC"),
    )
    .unwrap();

    let charizard_pok = poks.iter().find(|i| i.id == "CHARIZARD").unwrap();
    let charizard = PokemonInstance::new(
      charizard_pok,
      Level {
        level: 18,
        a_half: true,
      },
      11,
      8,
      15,
      "FIRE_SPIN_FAST",
      "FIRE_BLAST",
      Some("DRAGON_CLAW"),
    )
    .unwrap();

    assert_eq!(altaria.cp(), 1500);
    assert_eq!(noctowl.cp(), 1491);
    assert_eq!(charizard.cp(), 1473);
  }
}
