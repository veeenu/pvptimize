use crate::model::PokemonInstance;
use crate::model::moves::Damage;

pub trait StateMachine<D> {
  fn transition(self: &Self, env: D) -> Self;
}

#[derive(Debug, Copy, Clone)]
enum ChargedChoice {
  Main, Other
}

#[derive(Debug, Copy, Clone)]
pub enum MoveStateMachine {
  Neutral,
  Idle(i32),
  RegisterFast,
  RegisterCharged(ChargedChoice)
}

// TODO: consider also the other pokemon
impl<'a> StateMachine<(&'a PokemonState<'a>, &'a PokemonState<'a>)> for MoveStateMachine {
  fn transition(&self, env: (&'a PokemonState<'a>, &'a PokemonState<'a>)) -> Self {
    let (pok, opponent) = env;
    match self {
      MoveStateMachine::Neutral => {
        if ((pok.energy as i16) + pok.pokemon.charged_move2.energy >= 0) &&
            (
              pok.would_charged_kill(ChargedChoice::Other, opponent) ||
              opponent.shields != Shields::None ||
              true // TODO "The opponent's next action would result in a KO"
            )
        {
          MoveStateMachine::RegisterCharged(ChargedChoice::Other)
        } else if (pok.energy as i16) + pok.pokemon.charged_move1.energy >= 0 {
          MoveStateMachine::RegisterCharged(ChargedChoice::Main)
        } else {
          MoveStateMachine::Idle(pok.pokemon.fast_move.turns).transition(env)
        }
      },
      MoveStateMachine::Idle(0) => MoveStateMachine::RegisterFast,
      MoveStateMachine::Idle(i) => MoveStateMachine::Idle(i - 1),
      MoveStateMachine::RegisterFast => MoveStateMachine::Neutral.transition(env),
      MoveStateMachine::RegisterCharged(_) => MoveStateMachine::Neutral.transition(env)
    }
  }
}

#[derive(PartialEq, Eq, Debug)]
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
      Shields::None => Shields::None // BRB
    }
  }
}

impl Shields {
  fn available(&self) -> bool {
    *self != Shields::None
  }
}

pub struct PokemonState<'a> {
  pokemon: &'a PokemonInstance<'a>,
  health: i32,
  energy: i32,
  shields: Shields,
  state: MoveStateMachine,
}

impl<'a> PokemonState<'a> {
  fn new(
    pokemon  : &'a PokemonInstance<'a>,
    shields  : Shields
  ) -> PokemonState<'a> {
    PokemonState {
      pokemon,
      shields,
      health: pokemon.stamina() as _,
      energy: 0,
      state: MoveStateMachine::Neutral
    }
  }

  fn defend_charged(&mut self, damage: i32) {
    match self.shields {
      Shields::None => self.health = i32::max(0, self.health as i32 - damage) as _,
      _ => self.shields = self.shields.transition(())
    }
  }

  // TODO returns "is other dead?", use a dedicated type for clarity
  fn register_fast(&mut self, opponent: &mut PokemonState<'a>) -> bool { 
    let health = opponent.health as i32;
    let damage = self.pokemon.fast_move.calculate(self.pokemon, opponent.pokemon);
    let energy = self.pokemon.fast_move.energy;

    opponent.health = i32::max(0, health - damage) as _;
    self.energy += energy as i32;

    opponent.health == 0
  }

  fn register_charged(&mut self, choice: ChargedChoice, opponent: &mut PokemonState<'a>) -> bool {
    let charged_move = match choice {
      ChargedChoice::Main => self.pokemon.charged_move1,
      ChargedChoice::Other => self.pokemon.charged_move2,
    };

    let health = opponent.health as i32;
    let damage = charged_move.calculate(self.pokemon, opponent.pokemon);
    let energy_expenditure = charged_move.energy as i32;
    let current_energy = self.energy as i32;

    opponent.defend_charged(damage);
    self.energy = i32::max(0, current_energy + energy_expenditure) as _;

    opponent.health == 0
  }

  fn would_charged_kill(&self, choice: ChargedChoice, opponent: &PokemonState<'a>) -> bool {
    let charged_move = match choice {
      ChargedChoice::Main => self.pokemon.charged_move1,
      ChargedChoice::Other => self.pokemon.charged_move2,
    };
    let damage = charged_move.calculate(self.pokemon, opponent.pokemon);
    damage > opponent.health
  }

  fn transition(&mut self, other: &PokemonState<'a>) -> MoveStateMachine {
    self.state = self.state.transition((self, other));
    self.state
  }
}

#[derive(Debug)]
pub enum BattleOutcome {
  Continue,
  Win,
  Loss,
  Draw,
}

pub struct Battle<'a> {
  pokemon1: PokemonState<'a>,
  pokemon2: PokemonState<'a>,
  turn: u16
}

impl<'a> Battle<'a> {
  pub fn new(
    pokemon1: &'a PokemonInstance<'a>,
    pokemon2: &'a PokemonInstance<'a>,
    shields1: Shields,
    shields2: Shields,
  ) -> Battle<'a> {
    Battle {
      pokemon1: PokemonState::new(pokemon1, shields1),
      pokemon2: PokemonState::new(pokemon2, shields2),
      turn: 0
    }
  }

  pub fn turn(&mut self) -> BattleOutcome {
    let old_state1 = self.pokemon1.state;
    let old_state2 = self.pokemon2.state;
    let new_state1 = self.pokemon1.transition(&self.pokemon2);
    let new_state2 = self.pokemon2.transition(&self.pokemon1);

    {
      let p1 = &self.pokemon1;
      let p2 = &self.pokemon2;
      println!("\n  === Turn {:>5} ===", self.turn);
      println!("{:?} -> {:?} H {:>4} E {:>4} {:?}", old_state1, new_state1, p1.health, p1.energy, p1.shields);
      println!("{:?} -> {:?} H {:>4} E {:>4} {:?}", old_state2, new_state2, p2.health, p2.energy, p2.shields);
    }

    match (new_state1, new_state2) {
      (MoveStateMachine::RegisterCharged(choice1), MoveStateMachine::RegisterCharged(choice2)) => {
        // CMP Tie
        if self.pokemon1.pokemon.attack() > self.pokemon2.pokemon.attack() {
          self.pokemon1.register_charged(choice1, &mut self.pokemon2);
          self.pokemon2.register_charged(choice2, &mut self.pokemon1); // TODO block if it has been killed
        } else {
          self.pokemon2.register_charged(choice2, &mut self.pokemon1);
          self.pokemon1.register_charged(choice1, &mut self.pokemon2); // TODO block if it has been killed
        }
      },
      (MoveStateMachine::RegisterCharged(choice), MoveStateMachine::Idle(_)) => {
        self.pokemon1.register_charged(choice, &mut self.pokemon2);
      },
      (MoveStateMachine::RegisterCharged(choice), MoveStateMachine::RegisterFast) => {
        self.pokemon1.register_charged(choice, &mut self.pokemon2);
        self.pokemon2.register_fast(&mut self.pokemon1);
      },
      (MoveStateMachine::Idle(_), MoveStateMachine::RegisterCharged(choice)) => {
        self.pokemon2.register_charged(choice, &mut self.pokemon1);
      },
      (MoveStateMachine::RegisterFast, MoveStateMachine::RegisterCharged(choice)) => {
        self.pokemon2.register_charged(choice, &mut self.pokemon1);
        self.pokemon1.register_fast(&mut self.pokemon2);
      },
      (MoveStateMachine::RegisterFast, MoveStateMachine::Idle(_)) => {
        self.pokemon1.register_fast(&mut self.pokemon2);
      },
      (MoveStateMachine::Idle(_), MoveStateMachine::RegisterFast) => {
        self.pokemon2.register_fast(&mut self.pokemon1);
      },
      (MoveStateMachine::RegisterFast, MoveStateMachine::RegisterFast) => {
        self.pokemon1.register_fast(&mut self.pokemon2);
        self.pokemon2.register_fast(&mut self.pokemon1);
      },
      (MoveStateMachine::Idle(_), MoveStateMachine::Idle(_)) => {},
      (MoveStateMachine::Neutral, _) => unreachable!(),
      (_, MoveStateMachine::Neutral) => unreachable!(),
    }

    self.turn += 1;

    let outcome = match (self.pokemon1.health, self.pokemon2.health) {
      (0, 0) => BattleOutcome::Draw,
      (0, a) if a > 0 => BattleOutcome::Loss,
      (a, 0) if a > 0 => BattleOutcome::Win,
      (_, _) => BattleOutcome::Continue
    };

    println!("{:?}", outcome);

    outcome
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
    // https://pvpoke.com/battle/1500/victreebel-23-1-15-15-4-4-1/whiscash-28-0-14-13-4-4-1/22/1-2-1/0-2-1/
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
      "LEAF_BLADE",
      Some("ACID_SPRAY")
    ).unwrap();

    let whiscash = poks.iter().find(|i| i.id == "WHISCASH").unwrap();
    let whiscash = PokemonInstance::new(
      whiscash,
      Level { level: 28, a_half: false },
      0, 14, 13,
      "MUD_SHOT_FAST",
      "BLIZZARD",
      Some("MUD_BOMB"),
    ).unwrap();

    let mut battle = Battle::new(&victreebel, &whiscash, Shields::Two, Shields::Two);

    loop {
      match battle.turn() {
        BattleOutcome::Continue => {},
        _ => break
      }
    }
  }

  #[test]
  fn test_lucario_mirror() {
    // https://pvpoke.com/battle/1500/lucario-21-15-0-0-4-4-1/lucario-20.5-0-15-15-4-4-1/22/1-1-5/1-1-5/
    let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();

    let mech = Mechanics::new(&gm).unwrap();
    let poks = mech.pokemon().unwrap();

    let lucario_data = poks.iter().find(|i| i.id == "LUCARIO").unwrap();

    let lucario_attacker = PokemonInstance::new(
      lucario_data,
      Level { level: 21, a_half: false },
      15, 0, 0,
      "COUNTER_FAST",
      "AURA_SPHERE",
      Some("SHADOW_BALL"),
    ).unwrap();

    let lucario_defender = PokemonInstance::new(
      lucario_data,
      Level { level: 20, a_half: true },
      0, 15, 15,
      "COUNTER_FAST",
      "AURA_SPHERE",
      Some("SHADOW_BALL"),
    ).unwrap();

    assert_eq!(lucario_attacker.cp(), 1480);
    assert_eq!(lucario_defender.cp(), 1488);

    let mut battle = Battle::new(&lucario_attacker, &lucario_defender, Shields::Two, Shields::Two);

    loop {
      match battle.turn() {
        BattleOutcome::Continue => {},
        _ => break
      }
    }

    println!("{} {} | {} {}",
      battle.pokemon1.health,
      battle.pokemon1.energy,
      battle.pokemon2.health,
      battle.pokemon2.energy,
    );
  }

  #[test]
  fn test_registeel_mirror() {
    // https://pvpoke.com/battle/1500/registeel-22.5-15-2-5-4-4-1/registeel-24.5-1-12-1-4-4-1/22/0-1-2/0-1-2/
    let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();

    let mech = Mechanics::new(&gm).unwrap();
    let poks = mech.pokemon().unwrap();

    let regi = poks.iter().find(|i| i.id == "REGISTEEL").unwrap();
    let regi1 = PokemonInstance::new(
      regi,
      Level { level: 22, a_half: true },
      15, 2, 5,
      "LOCK_ON_FAST",
      "FLASH_CANNON",
      Some("FOCUS_BLAST")
    ).unwrap();

    let regi2 = PokemonInstance::new(
      regi,
      Level { level: 24, a_half: true },
      1, 12, 1,
      "LOCK_ON_FAST",
      "FLASH_CANNON",
      Some("FOCUS_BLAST")
    ).unwrap();

    let mut battle = Battle::new(&regi1, &regi2, Shields::Two, Shields::Two);

    loop {
      match battle.turn() {
        BattleOutcome::Continue => {},
        _ => break
      }
    }

    println!("{} {} | {} {}",
      battle.pokemon1.health,
      battle.pokemon1.energy,
      battle.pokemon2.health,
      battle.pokemon2.energy,
    );
  }
}
