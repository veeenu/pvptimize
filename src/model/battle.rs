use crate::model::PokemonInstance;
use crate::model::moves::Damage;

pub trait StateMachine<D> {
  fn transition(self: Self, env: D) -> Self;
}

pub enum Turns {
  T5, T4, T3, T2, T1
}

pub enum MoveSM {
  Neutral,
  Idle(Turns),
  RegisterFast,
  RegisterCharged
}

#[derive(PartialEq, Eq)]
pub enum Shields {
  Two,
  One,
  None
}

impl StateMachine<()> for Shields {
  fn transition(self, _: ()) -> Self {
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
  base_damages: BaseDamages
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
      base_damages: BaseDamages::new(pokemon1, pokemon2)
    }
  }
}