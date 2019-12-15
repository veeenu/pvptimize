use serde;
use serde::{Deserialize};
use serde_json;

#[derive(Deserialize, Debug)]
pub struct AvatarCustomization {
  enabled: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged, rename_all = "camelCase")]
pub enum PvPMove {
  FastMove {
    unique_id: String,
    #[serde(rename = "type")]
    type_: String,
    power: f64,
    vfx_name: String,
    duration_turns: u16,
    energy_delta: u16,
  },
  ChargedMove {
    unique_id: String,
    #[serde(rename = "type")]
    type_: String,
    power: f64,
    vfx_name: String,
    energy_delta: i16,
  },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FormDetail {
  form: String,
  asset_bundle_suffix: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct Form {
  pokemon: String,
  forms: Vec<FormDetail>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLevel {
  cp_multiplier: Vec<f64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TypeEffectiveness {
  attack_type: String,
  #[serde(rename = "attackScalar")]
  effectiveness: Vec<f64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
  base_attack: u8,
  base_defense: u8,
  base_stamina: u8,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThirdMove {
  stardust_to_unlock: u64,
  candy_to_unlock: u64
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PokemonSettings {
  pokemon_id: String,
  family_id: String,
  #[serde(rename = "type")]
  type1: String,
  type2: Option<String>,
  stats: Stats,
  quick_moves: Vec<String>,
  cinematic_moves: Vec<String>,
  third_move: ThirdMove,
  candy_to_evolve: u64,
  form: String
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
struct ItemTemplate {
  template_id: String,
  #[serde(flatten)]
  entry: Option<GameMasterEntry>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameMaster {
  item_templates: Vec<ItemTemplate>
}

#[cfg(test)]
mod test {

  use super::*;

  #[test]
  fn test() {
    let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();

    for n in gm.item_templates.iter() {
      if n.entry.is_some() {
        println!("{:?}", n.entry);
      }

      /*match &n.unwrap().entry {
        Some(GameMasterEntry::PvPMove(c)) => {
          println!("{:?}", c);
          ca += 1;
        }
        Some(GameMasterEntry::Form(c)) => {
          println!("{:?}", c);
          cb += 1;
        }
        _ => {}
      }*/
    }
  }
}
