use crate::model::{Type, TYPE_ORDERING};
use serde;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AvatarCustomization {
  enabled: Option<bool>,
}

// Untagged bugs out here
/*#[derive(Deserialize, Debug)]
#[serde(untagged, rename_all = "camelCase")]
pub enum PvPMove {
  #[serde(rename_all = "camelCase")]
  FastMove {
    unique_id: String,
    #[serde(rename = "type")]
    type_: String,
    power: f64,
    vfx_name: String,
    duration_turns: u16,
    energy_delta: u16,
  },
  #[serde(rename_all = "camelCase")]
  ChargedMove {
    unique_id: String,
    #[serde(rename = "type")]
    type_: String,
    power: f64,
    vfx_name: String,
    energy_delta: i16,
  },
}*/

fn pvp_move_default_duration_turns() -> u16 {
  1
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PvPMove {
  pub unique_id: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub power: f64,
  pub vfx_name: String,
  #[serde(default = "pvp_move_default_duration_turns")]
  // Sometimes, like in the DRAGON_BREATH case, it is absent
  pub duration_turns: u16,
  pub energy_delta: i16,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FormDetail {
  form: String,
  asset_bundle_suffix: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Form {
  pokemon: String,
  forms: Vec<FormDetail>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLevel {
  pub cp_multiplier: Vec<f64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TypeEffectiveness {
  pub attack_type: String,
  #[serde(rename = "attackScalar")]
  pub effectiveness: Vec<f64>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
  pub base_attack: u8,
  pub base_defense: u8,
  pub base_stamina: u8,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThirdMove {
  stardust_to_unlock: u64,
  candy_to_unlock: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PokemonSettings {
  pub pokemon_id: String,
  family_id: String,
  #[serde(rename = "type")]
  pub type1: String,
  pub type2: Option<String>,
  pub stats: Stats,
  pub quick_moves: Vec<String>,
  pub cinematic_moves: Vec<String>,
  third_move: ThirdMove,
  candy_to_evolve: Option<u64>,
  pub form: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PokemonUpgrades {
  candy_cost: Vec<u16>,
  stardust_cost: Vec<u16>,
  shadow_stardust_multiplier: f64,
  shadow_candy_multiplier: f64,
  purified_stardust_multiplier: f64,
  purified_candy_multiplier: f64,
}

#[derive(Deserialize, Debug)]
pub enum GameMasterEntry {
  #[serde(rename = "avatarCustomization")]
  AvatarCustomization(AvatarCustomization),
  #[serde(rename = "combatMove")]
  PvPMove(PvPMove),
  #[serde(rename = "formSettings")]
  Form(Form),
  #[serde(rename = "playerLevel")]
  PlayerLevel(PlayerLevel),
  #[serde(rename = "pokemonSettings")]
  PokemonSettings(PokemonSettings),
  #[serde(rename = "pokemonUpgrades")]
  PokemonUpgrades(PokemonUpgrades),
  #[serde(rename = "typeEffective")]
  TypeEffectiveness(TypeEffectiveness),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemTemplate {
  template_id: String,
  #[serde(flatten)]
  pub entry: Option<GameMasterEntry>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameMaster {
  pub item_templates: Vec<ItemTemplate>,
}

#[cfg(test)]
mod test {

  use super::*;
  use crate::model::Mechanics;

  #[test]
  fn test() {
    let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();

    let mech = Mechanics::new(&gm).unwrap();
    let steel_psychic = mech.dual_type_effectiveness(Type::Steel, Type::Psychic);

    assert!((steel_psychic[&Type::Poison] - 0.391).abs() < 10e-3);
    assert!((steel_psychic[&Type::Psychic] - 0.391).abs() < 10e-3);
    assert!((steel_psychic[&Type::Ghost] - 1.6).abs() < 10e-3);
    assert!((steel_psychic[&Type::Fighting] - 1.).abs() < 10e-3);

    for k in TYPE_ORDERING.iter() {
      println!("{:>15} {:.3}", format!("{:?}", k), steel_psychic[k]);
    }
  }
}
