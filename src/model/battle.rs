use crate::model::PokemonInstance;
use crate::model::moves::Damage;

use std::convert::TryFrom;

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
impl<'a> StateMachine<(&TurnState<'a>, &TurnState<'a>)> for MoveStateMachine {
  fn transition(&self, env: (&TurnState<'a>, &TurnState<'a>)) -> Self {
    let (attacker, defender) = env;
    match self {
      MoveStateMachine::Neutral => {
        // TODO logic to implement in this branch:
        // - Shield bait logic:
        //   if energy >= best move energy cost:
        //     if best move energy cost > other move energy cost:
        //       if opponent has shields:
        //         do other move
        //       else:
        //         do best move
        //     else:
        //       do best move
        //   else:
        //     do fast move
        // - Fast move priority: if fast move kills opponent, choose it first
        let dpe_main: f64 =
          attacker.instance.charged_move1.calculate(attacker.instance, defender.instance) as f64 /
          -attacker.instance.charged_move1.energy as f64;
        let dpe_other: f64 =
          attacker.instance.charged_move2.calculate(attacker.instance, defender.instance) as f64 /
          -attacker.instance.charged_move2.energy as f64;

        let (best_move, other_move) = if dpe_main > dpe_other {
          (
            (&attacker.instance.charged_move1, ChargedChoice::Main),
            (&attacker.instance.charged_move2, ChargedChoice::Other),
          )
        } else {
          (
            (&attacker.instance.charged_move2, ChargedChoice::Other),
            (&attacker.instance.charged_move1, ChargedChoice::Main),
          )
        };

        if attacker.state.energy + best_move.0.energy >= 0 {
          if (best_move.0.energy.abs() >= other_move.0.energy.abs()) && (defender.state.shields != Shields::None) {
            MoveStateMachine::RegisterCharged(other_move.1)
          } else {
            MoveStateMachine::RegisterCharged(best_move.1)
          }
        } else {
          MoveStateMachine::RegisterFast
        }

        /*if (attacker.state.energy + attacker.instance.charged_move2.energy) >= 0 &&
           (
             (attacker.would_charged_kill(ChargedChoice::Other, defender) && defender.state.shields == Shields::None) ||
             // defender.state.shields != Shields::None || // Shield baiting
             false // TODO "The opponent's next action would result in a KO"
           )
        {
          MoveStateMachine::RegisterCharged(ChargedChoice::Other)
        } else if (attacker.state.energy + attacker.instance.charged_move1.energy) >= 0 {
          MoveStateMachine::RegisterCharged(ChargedChoice::Main)
        } else {
          MoveStateMachine::Idle(attacker.instance.fast_move.turns).transition(env)
        }*/
      },
      MoveStateMachine::Idle(0) => MoveStateMachine::RegisterFast,
      MoveStateMachine::Idle(i) => MoveStateMachine::Idle(i - 1),
      MoveStateMachine::RegisterFast => MoveStateMachine::Neutral.transition(env),
      MoveStateMachine::RegisterCharged(_) => MoveStateMachine::Neutral.transition(env)
    }
  }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
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

#[derive(Copy, Clone)]
pub struct PokemonState {
  health: i16,
  energy: i16,
  shields: Shields,
  state: MoveStateMachine,
}

impl PokemonState {
  fn new(
    pokemon  : &PokemonInstance,
    shields  : Shields
  ) -> PokemonState {
    PokemonState {
      health: pokemon.stamina() as _,
      energy: 0,
      shields,
      state: MoveStateMachine::Neutral
    }
  }
}

impl std::fmt::Display for PokemonState {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "[H: {:>3} E: {:>3} S: {:1} ({:?})]",
      self.health,
      self.energy,
      match self.shields {
        Shields::None => 0,
        Shields::One => 1,
        Shields::Two => 2,
      },
      self.state
    )
  }
}

pub struct TurnState<'a> {
  state: PokemonState,
  instance: &'a PokemonInstance,
}

impl<'a> TurnState<'a> {
  pub fn new(
    state: PokemonState,
    instance: &'a PokemonInstance,
  ) -> TurnState<'a> {
    TurnState {
      state, instance,
    }
  }
  
  // TODO returns "is other dead?", use a dedicated type for clarity
  fn register_fast(&mut self, opponent: &TurnState<'a>) -> i16 { 
    let damage = self.instance.fast_move.calculate(&self.instance, &opponent.instance);
    let energy = self.instance.fast_move.energy;

    self.state.energy += energy;
    // self.defend_fast(damage)
    damage
  }

  fn register_charged(&mut self, choice: ChargedChoice, opponent: &TurnState<'a>) -> i16 {
    let charged_move = match choice {
      ChargedChoice::Main => &self.instance.charged_move1,
      ChargedChoice::Other => &self.instance.charged_move2,
    };

    let damage = charged_move.calculate(&self.instance, &opponent.instance);
    let energy_expenditure = charged_move.energy;
    let current_energy = self.state.energy;

    self.state.energy = current_energy + energy_expenditure;
    assert!(self.state.energy >= 0);
    println!("{} {}", current_energy, energy_expenditure);

    // self.defend_charged(damage)
    damage
  }

  fn defend_fast(&mut self, damage: i16) -> bool {
    self.state.health = i16::max(0, self.state.health - damage) as _;
    self.state.health == 0
  }

  fn defend_charged(&mut self, damage: i16) -> bool {
    match self.state.shields {
      Shields::None => self.state.health = i16::max(0, self.state.health as i16 - damage) as _,
      _ => self.state.shields = self.state.shields.transition(())
    };
    self.state.health == 0
  }

  fn would_charged_kill(&self, choice: ChargedChoice, opponent: &TurnState<'a>) -> bool {
    let charged_move = match choice {
      ChargedChoice::Main => &self.instance.charged_move1,
      ChargedChoice::Other => &self.instance.charged_move2,
    };
    let damage = charged_move.calculate(&self.instance, &opponent.instance);
    damage > opponent.state.health
  }

  fn wins_cmp_tie(&self, opponent: &TurnState<'a>) -> bool {
    self.instance.attack() > opponent.instance.attack()
  }

  fn transition(&self, opponent: &TurnState<'a>) -> MoveStateMachine {
    self.state.state.transition((self, opponent))
  }
}

#[derive(Copy, Clone)]
pub enum BattleState {
  Win,
  Loss,
  Draw,
  Continue(PokemonState, MoveStateMachine, PokemonState, MoveStateMachine)
}

impl std::fmt::Display for BattleState {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      BattleState::Win => write!(f, "Win"),
      BattleState::Loss => write!(f, "Loss"),
      BattleState::Draw => write!(f, "Draw"),
      BattleState::Continue(a, _, b, _) => write!(f, "({}, {})", a, b)
    }
  }
}

pub struct PokemonStateDTO {
  pokemon_id: String,
  action: String,
  health: i16,
  energy: i16,
  shields: i32,
}

pub struct TurnStateDTO {
  attacker: PokemonStateDTO,
  defender: PokemonStateDTO,
}

impl From<(&PokemonInstance, &PokemonState, &MoveStateMachine)> for PokemonStateDTO {
  fn from((inst, state, mov): (&PokemonInstance, &PokemonState, &MoveStateMachine)) -> Self {
    PokemonStateDTO {
      pokemon_id: inst.pokemon.id.clone(),
      action: match mov {
        MoveStateMachine::RegisterFast => format!("uses fast move {}", inst.fast_move.uid),
        MoveStateMachine::RegisterCharged(ChargedChoice::Main) => format!("uses charged move {}", inst.charged_move1.uid),
        MoveStateMachine::RegisterCharged(ChargedChoice::Other) => format!("uses charged move {}", inst.charged_move2.uid),
        MoveStateMachine::Idle(_) => format!("waits"),
        MoveStateMachine::Neutral => format!("WTF")
      },
      health: state.health,
      energy: state.energy,
      shields: match state.shields {
        Shields::None => 0,
        Shields::One => 1,
        Shields::Two => 2
      }
    }
  }
}

impl std::fmt::Display for PokemonStateDTO {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{} [H {:>3} E {:>3} S {:1}] {}", self.pokemon_id, self.health, self.energy, self.shields, self.action)
  }
}

impl std::fmt::Display for TurnStateDTO {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}\n{}\n\n", self.attacker, self.defender)
  }
}

pub struct Battle {
  pokemon_instances: (PokemonInstance, PokemonInstance),
  turn: u16,
  state: BattleState
}

impl Battle {
  pub fn new(
    pokemon1: PokemonInstance,
    pokemon2: PokemonInstance,
    shields1: Shields,
    shields2: Shields,
  ) -> Battle {
    Battle {
      state: BattleState::Continue(
        PokemonState::new(&pokemon1, shields1),
        MoveStateMachine::Neutral,
        PokemonState::new(&pokemon2, shields2),
        MoveStateMachine::Neutral,
      ),
      pokemon_instances: (
        pokemon1,
        pokemon2,
      ),
      turn: 0
    }
  }
}

impl<'a> Iterator for Battle {
  type Item = TurnStateDTO;

  fn next(&mut self) -> Option<TurnStateDTO> {
    if let BattleState::Continue(
      state_attacker, move_state_attacker,
      state_defender, move_state_defender
    ) = self.state {
      let mut pokemon1 = TurnState::new(
        state_attacker, &self.pokemon_instances.0,
      );
      let mut pokemon2 = TurnState::new(
        state_defender, &self.pokemon_instances.1
      );

      let new_state1 = pokemon1.transition(&pokemon2);
      let new_state2 = pokemon2.transition(&pokemon1);

      {
        let p1 = &state_attacker;
        let p2 = &state_defender;
        println!("\n  === Turn {:>5} ===", self.turn);
        println!("{:?} -> {:?} H {:>4} E {:>4} {:?}", move_state_attacker, new_state1, p1.health, p1.energy, p1.shields);
        println!("{:?} -> {:?} H {:>4} E {:>4} {:?}", move_state_defender, new_state2, p2.health, p2.energy, p2.shields);
      };

      match (new_state1, new_state2) {
        (MoveStateMachine::RegisterCharged(choice1), MoveStateMachine::RegisterCharged(choice2)) => {
          // CMP Tie
          if pokemon1.wins_cmp_tie(&pokemon2) {
            pokemon2.defend_charged(pokemon1.register_charged(choice1, &pokemon2));
            pokemon1.defend_charged(pokemon2.register_charged(choice2, &pokemon1)); // TODO block if it has been killed
          } else {
            pokemon1.defend_charged(pokemon2.register_charged(choice2, &pokemon1));
            pokemon2.defend_charged(pokemon1.register_charged(choice1, &pokemon2)); // TODO block if it has been killed
          }
        },
        (MoveStateMachine::RegisterCharged(choice), MoveStateMachine::Idle(_)) => {
          pokemon2.defend_charged(pokemon1.register_charged(choice, &pokemon2));
        },
        (MoveStateMachine::RegisterCharged(choice), MoveStateMachine::RegisterFast) => {
          pokemon2.defend_charged(pokemon1.register_charged(choice, &pokemon2));
          pokemon1.defend_fast(pokemon2.register_fast(&pokemon1));
        },
        (MoveStateMachine::Idle(_), MoveStateMachine::RegisterCharged(choice)) => {
          pokemon1.defend_charged(pokemon2.register_charged(choice, &pokemon1));
        },
        (MoveStateMachine::RegisterFast, MoveStateMachine::RegisterCharged(choice)) => {
          pokemon1.defend_charged(pokemon2.register_charged(choice, &pokemon1));
          pokemon2.defend_fast(pokemon1.register_fast(&pokemon2));
        },
        (MoveStateMachine::RegisterFast, MoveStateMachine::Idle(_)) => {
          pokemon2.defend_fast(pokemon1.register_fast(&pokemon2));
        },
        (MoveStateMachine::Idle(_), MoveStateMachine::RegisterFast) => {
          pokemon2.register_fast(&pokemon1);
        },
        (MoveStateMachine::RegisterFast, MoveStateMachine::RegisterFast) => {
          pokemon2.defend_fast(pokemon1.register_fast(&pokemon2));
          pokemon1.defend_fast(pokemon2.register_fast(&pokemon1));
        },
        (MoveStateMachine::Idle(_), MoveStateMachine::Idle(_)) => {},
        (MoveStateMachine::Neutral, _) => unreachable!(),
        (_, MoveStateMachine::Neutral) => unreachable!(),
      }

      self.turn += 1;
      pokemon1.state.state = new_state1;
      pokemon2.state.state = new_state2;

      self.state = match (pokemon1.state.health, pokemon2.state.health) {
        (0, 0) => BattleState::Draw,
        (0, a) if a > 0 => BattleState::Loss,
        (a, 0) if a > 0 => BattleState::Win,
        (_, _) => BattleState::Continue(pokemon1.state, new_state1, pokemon2.state, new_state2)
      };

      // Some(self.state)
      Some(TurnStateDTO {
        attacker: (&self.pokemon_instances.0, &state_attacker, &new_state1).into(),
        defender: (&self.pokemon_instances.1, &state_defender, &new_state2).into(),
      })
    } else {
      None
    }
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
    // let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    // let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();
    // let mech = Mechanics::try_from(gm).unwrap();
    let mech = Mechanics::instance();

    let victreebel = mech.pokemon_instance(
      "VICTREEBEL",
      Level { level: 23, a_half: false },
      1, 15, 15,
      "RAZOR_LEAF_FAST",
      "LEAF_BLADE",
      Some("ACID_SPRAY"),
    ).unwrap();

    let whiscash = mech.pokemon_instance(
      "WHISCASH",
      Level { level: 28, a_half: false },
      0, 14, 13,
      "MUD_SHOT_FAST",
      "BLIZZARD",
      Some("MUD_BOMB"),
    ).unwrap();

    let battle = Battle::new(victreebel, whiscash, Shields::Two, Shields::Two);

    let v: Vec<_> = battle.collect();
    for (i, turn) in v.iter().enumerate() {
      println!("{:>4}\n{}", i, turn);
    }
  }

  #[test]
  fn test_lucario_mirror() {
    // https://pvpoke.com/battle/1500/lucario-21-15-0-0-4-4-1/lucario-20.5-0-15-15-4-4-1/22/1-1-5/1-1-5/
    // let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    // let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();
    // let mech = Mechanics::try_from(gm).unwrap();
    let mech = Mechanics::instance();

    let lucario_attacker = mech.pokemon_instance(
      "LUCARIO",
      Level { level: 21, a_half: false },
      15, 0, 0,
      "COUNTER_FAST",
      "AURA_SPHERE",
      Some("SHADOW_BALL"),
    ).unwrap();

    let lucario_defender = mech.pokemon_instance(
      "LUCARIO",
      Level { level: 20, a_half: true },
      0, 15, 15,
      "COUNTER_FAST",
      "AURA_SPHERE",
      Some("SHADOW_BALL"),
    ).unwrap();

    assert_eq!(lucario_attacker.cp(), 1480);
    assert_eq!(lucario_defender.cp(), 1488);

    let battle = Battle::new(lucario_attacker, lucario_defender, Shields::Two, Shields::Two);

    let v: Vec<_> = battle.collect();
    for (i, turn) in v.iter().enumerate() {
      println!("{:>4}\n{}", i, turn);
    }
  }

  #[test]
  fn test_registeel_mirror() {
    // https://pvpoke.com/battle/1500/registeel-22.5-15-2-5-4-4-1/registeel-24.5-1-12-1-4-4-1/22/0-1-2/0-1-2/
    // let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    // let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();
    // let mech = Mechanics::try_from(gm).unwrap();
    let mech = Mechanics::instance();

    let regi1 = mech.pokemon_instance(
      "REGISTEEL",
      Level { level: 22, a_half: true },
      15, 2, 5,
      "LOCK_ON_FAST",
      "FOCUS_BLAST",
      Some("FLASH_CANNON")
    ).unwrap();

    let regi2 = mech.pokemon_instance(
      "REGISTEEL",
      Level { level: 24, a_half: true },
      1, 12, 1,
      "LOCK_ON_FAST",
      "FOCUS_BLAST",
      Some("FLASH_CANNON")
    ).unwrap();

    println!("{:?}", regi1.type_effectiveness(&regi1.charged_move1));

    let battle = Battle::new(regi1, regi2, Shields::Two, Shields::Two);

    let v: Vec<_> = battle.collect();
    for (i, turn) in v.iter().enumerate() {
      println!("{:>4}\n{}", i, turn);
    }
  }
}
