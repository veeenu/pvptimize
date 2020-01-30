use crate::model::PokemonInstance;
use crate::model::moves::Damage;

pub trait StateMachine<D> {
  fn transition(self: &Self, env: D) -> Self;
}

/*pub enum Turns {
  T5, T4, T3, T2, T1
}*/

#[derive(Debug)]
pub enum MoveSM {
  Neutral,
  Idle(u16),
  RegisterFast,
  RegisterCharged
}

// TODO: consider also the other pokemon
impl<'a> StateMachine<&'a BattlingPokemon<'a>> for MoveSM {
  fn transition(&self, pok: &'a BattlingPokemon<'a>) -> Self {
    match self {
      MoveSM::Neutral => {
        if (pok.energy as i16) + pok.pokemon.charged_move1.energy > 0 {
          MoveSM::RegisterCharged
        } else {
          MoveSM::Idle(pok.pokemon.fast_move.turns).transition(pok)
        }
      },
      MoveSM::Idle(0) => MoveSM::RegisterFast,
      MoveSM::Idle(i) => MoveSM::Idle(i - 1),
      MoveSM::RegisterFast => MoveSM::Neutral.transition(pok),
      MoveSM::RegisterCharged => MoveSM::Neutral.transition(pok)
    }
  }
}

#[derive(PartialEq, Eq)]
pub enum Shields {
  Two,
  One,
  None
}

impl StateMachine<()> for Shields {
  fn transition(&self, _: ()) -> Self {
    match self {
      Shields::Two => Shields::One,
      Shields::One => Shields::None,
      Shields::None => Shields::None
    }
  }
}

impl Shields {
  fn available(&self) -> bool {
    *self != Shields::None
  }
}

struct BaseDamages {
  fast1v2: u16,
  charged_main1v2: u16,
  charged_other1v2: Option<u16>,
  fast2v1: u16,
  charged_main2v1: u16,
  charged_other2v1: Option<u16>,
}

impl BaseDamages {
  fn new(pok1: &PokemonInstance<'_>, pok2: &PokemonInstance<'_>) -> BaseDamages {
    BaseDamages {
      fast1v2: pok1.fast_move.calculate(pok1, pok2),
      charged_main1v2: pok1.charged_move1.calculate(pok1, pok2),
      charged_other1v2: pok1.charged_move2.map(|m| m.calculate(pok1, pok2)),
      fast2v1: pok2.fast_move.calculate(pok2, pok1),
      charged_main2v1: pok2.charged_move1.calculate(pok2, pok1),
      charged_other2v1: pok2.charged_move2.map(|m| m.calculate(pok2, pok1)),
    }
  }
}

pub struct BattlingPokemon<'a> {
  pokemon: &'a PokemonInstance<'a>,
  health: u32,
  energy: u32,
  shields: Shields
}

impl<'a> BattlingPokemon<'a> {
  fn new(pokemon: &'a PokemonInstance<'a>, shields: Shields) -> BattlingPokemon<'a> {
    BattlingPokemon {
      pokemon,
      shields,
      health: pokemon.stamina() as _,
      energy: 0,
    }
  }
}

pub struct Battle<'a> {
  pokemon1: BattlingPokemon<'a>,
  pokemon2: BattlingPokemon<'a>,
  base_damages: BaseDamages,
  state1: MoveSM,
  state2: MoveSM,
}

impl<'a> Battle<'a> {
  pub fn new(
    pokemon1: &'a PokemonInstance<'a>,
    pokemon2: &'a PokemonInstance<'a>,
    shields1: Shields,
    shields2: Shields,
  ) -> Battle<'a> {
    Battle {
      pokemon1: BattlingPokemon::new(pokemon1, shields1),
      pokemon2: BattlingPokemon::new(pokemon2, shields2),
      base_damages: BaseDamages::new(pokemon1, pokemon2),
      state1: MoveSM::Neutral,
      state2: MoveSM::Neutral,
    }
  }

  fn register_fast1(&mut self) {
    let health = self.pokemon2.health as i32;
    let damage = self.base_damages.fast1v2 as i32;
    let energy = self.pokemon1.pokemon.fast_move.energy as u32;

    self.pokemon2.health = i32::max(0, health - damage) as u32;
    self.pokemon1.energy += energy;
    // If pokemon 2 dead then...
  }
  fn register_fast2(&mut self) {
    let health = self.pokemon1.health as i32;
    let damage = self.base_damages.fast2v1 as i32;
    let energy = self.pokemon2.pokemon.fast_move.energy as u32;

    self.pokemon1.health = i32::max(0, health - damage) as u32;
    self.pokemon2.energy += energy;
    // If pokemon 1 dead then...
  }

  fn register_charged1(&mut self) {
    let health = self.pokemon2.health as i32;
    let damage = self.base_damages.charged_main1v2 as i32;
    let energy = self.pokemon1.pokemon.charged_move1.energy as i32;
    let pk_energy = self.pokemon1.energy as i32;

    self.pokemon2.health = i32::max(0, health - damage) as u32;
    self.pokemon1.energy = i32::max(0, pk_energy + energy) as u32;
    // If pokemon 2 dead then...
  }

  fn register_charged2(&mut self) {
    let health = self.pokemon1.health as i32;
    let damage = self.base_damages.charged_main2v1 as i32;
    let energy = self.pokemon2.pokemon.charged_move1.energy as i32;
    let pk_energy = self.pokemon2.energy as i32;

    self.pokemon1.health = i32::max(0, health - damage) as u32;
    self.pokemon2.energy = i32::max(0, pk_energy + energy) as u32;
    // If pokemon 1 dead then...
  }

  pub fn turn(&mut self) {
    let new_state1 = self.state1.transition(&self.pokemon1);
    let new_state2 = self.state2.transition(&self.pokemon2);

    println!("{:?} {:?}", new_state1, new_state2);

    match (new_state1, new_state2) {
      (MoveSM::RegisterCharged, MoveSM::RegisterCharged) => {
        // CMP Tie
        if self.pokemon1.pokemon.attack() > self.pokemon2.pokemon.attack() {
          self.register_charged1();
          self.register_charged2();
        } else {
          self.register_charged2();
          self.register_charged1();
        }
      },
      (MoveSM::RegisterCharged, MoveSM::Idle(_)) => {
        self.register_charged1();
      },
      (MoveSM::RegisterCharged, MoveSM::RegisterFast) => {
        self.register_charged1();
        self.register_fast2();
      },
      (MoveSM::Idle(_), MoveSM::RegisterCharged) => {
        self.register_charged2();
      },
      (MoveSM::RegisterFast, MoveSM::RegisterCharged) => {
        self.register_charged2();
        self.register_fast1();
      },
      (MoveSM::RegisterFast, MoveSM::Idle(_)) => {
        self.register_fast1();
      },
      (MoveSM::Idle(_), MoveSM::RegisterFast) => {
        self.register_fast2();
      },
      (MoveSM::RegisterFast, MoveSM::RegisterFast) => {
        self.register_fast1();
        self.register_fast2();
      },
      (MoveSM::Idle(_), MoveSM::Idle(_)) => {},
      (Neutral, _) => unreachable!(),
      (_, Neutral) => unreachable!(),
    }

    self.state1 = self.state1.transition(&self.pokemon1);
    self.state2 = self.state2.transition(&self.pokemon2);

    let p1 = &self.pokemon1;
    let p2 = &self.pokemon2;
    println!("\n  === Turn ===");
    println!("{:?} {} {}", self.state1, p1.health, p1.energy);
    println!("{:?} {} {}", self.state2, p2.health, p2.energy);
  }
}

#[cfg(test)]
mod tests {
  use crate::model::pokemon::Level;
  use crate::error::*;
  use crate::gamemaster::*;
  use crate::model::mechanics::*;
  use super::*;

  #[test]
  fn test_victreebel_vs_whiscash() {
    let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();

    let mech = Mechanics::new(&gm).unwrap();
    let poks = mech.pokemon().unwrap();

    let victreebel = poks.iter().find(|i| i.id == "VICTREEBEL").unwrap();
    let victreebel = PokemonInstance::new(
      victreebel,
      Level { level: 23, a_half: false },
      1, 15, 15,
      "RAZOR_LEAF_FAST",
      "SOLAR_BEAM",
      None
    ).unwrap();

    let whiscash = poks.iter().find(|i| i.id == "WHISCASH").unwrap();
    let whiscash = PokemonInstance::new(
      whiscash,
      Level { level: 28, a_half: false },
      0, 14, 13,
      "MUD_SHOT_FAST",
      "BLIZZARD",
      None
    ).unwrap();

    let mut battle = Battle::new(&victreebel, &whiscash, Shields::None, Shields::None);

    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();
    battle.turn();

  }
}