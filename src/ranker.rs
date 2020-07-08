use crate::model::{Level, Mechanics, PokemonInstance};

use std::collections::BTreeMap;

fn max_statproduct(mech: &Mechanics, pokemon_id: &str, cap: usize) -> Result<(i32, i32, i32, Level, u32), ()> {
  let pok = mech.pokemon(pokemon_id).ok_or_else(|| ())?;
  const MIN_LEVEL: Level = Level { level: 1, a_half: false };

  let (base_atk, base_def, base_sta): (f64, f64, f64) = 
    (pok.stats.base_attack as _, pok.stats.base_defense as _, pok.stats.base_stamina as _);

  let (max_atk, max_def, max_sta) =
    (base_atk + 15., base_def + 15., base_sta + 15.)
    as (f64, f64, f64);

  let max_cpm = (10f64 * cap as f64 / (base_atk * base_def.sqrt() * base_sta.floor().sqrt())).sqrt();
  let min_cpm = (10f64 * cap as f64 / (max_atk * max_def.sqrt() * max_sta.floor().sqrt())).sqrt();

  let cpms: Vec<(f64, Level)> = (0..79)
    .map(|i| Level::from(i))
    .map(|i| {
      (mech.cp_multiplier(&i), i)
    })
    .collect();

  let min_level = cpms.iter()
    .find(|(cpm, _)| cpm >= &min_cpm)
    .map(|&(_, level)| level)
    .unwrap_or_else(|| Level { level: 1, a_half: false });
  let max_level = cpms.iter()
    .rfind(|(cpm, _)| cpm <= &max_cpm)
    .map(|&(_, level)| level)
    .unwrap_or_else(|| Level { level: 40, a_half: false });

  let instance = iv_combinations.iter()
    .map(|&(atk, def, sta)| {
      let mut level = max_level;

      while level >= MIN_LEVEL {

        let cpm = mech.cp_multiplier(&level);
        let a = (pok.stats.base_attack + atk as u16) as f64;
        let d = (pok.stats.base_defense + def as u16) as f64;
        let s = (pok.stats.base_stamina + sta as u16) as f64;

        // There's a waste of computation around here
        let cp = f64::floor(a * d.sqrt() * s.floor().sqrt() * cpm * cpm / 10.) as u32;
        let stat_product = a * cpm * d * cpm * (s * cpm).floor() / 1000.;

        if cp <= cap as _ {
          return (atk, def, sta, level, stat_product)
        }
        level = level.prev();
      }

      (atk, def, sta, level, 0.) // should be unreachable
    })
    .fold((0, 0, 0, MIN_LEVEL, 0.), |max, cur| {
      if max.4 <= cur.4 {
        cur
      } else {
        max
      }
    });

  println!("{:?}", instance);

  Ok((
      instance.0,
      instance.1,
      instance.2,
      instance.3,
      instance.4.round() as u32
    ))
}

lazy_static! {
  static ref iv_combinations: Vec<(i32, i32, i32)> = {
    (0..=15).into_iter()
      .flat_map(move |i: i32|
        (0..=15).into_iter()
          .flat_map(move |j: i32|
            (0..=15).into_iter()
              .map(move |k| (i, j, k))))
              .collect()
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::gamemaster::GameMaster;

  use std::convert::TryFrom;

  #[test]
  fn test_iv_combs() {
    assert_eq!(super::iv_combinations.len(), 16*16*16);
  }

  #[test]
  fn test_max_statproduct() {
    let mech = Mechanics::instance();

    use std::time::Instant;

    let start = Instant::now();
    assert_eq!(
      max_statproduct(&mech, "ALTARIA", 1500).unwrap(),
      (0, 14, 15, Level { level: 29, a_half: false }, 2212)
    );
    let dur = Instant::now() - start;
    println!("{:?}", dur);

    let start = Instant::now();
    assert_eq!(
      max_statproduct(&mech, "WOBBUFFET", 1500).unwrap(),
      (15, 15, 15, Level { level: 40, a_half: false }, 1774)
    );
    let dur = Instant::now() - start;
    println!("{:?}", dur);

    let start = Instant::now();
    assert_eq!(
      max_statproduct(&mech, "BLISSEY", 1500).unwrap(),
      (0, 15, 3, Level { level: 21, a_half: true }, 2814)
    );
    let dur = Instant::now() - start;
    println!("{:?}", dur);

    let start = Instant::now();
    assert_eq!(
      max_statproduct(&mech, "GENGAR", 1500).unwrap(),
      (0, 13, 13, Level { level: 19, a_half: true }, 1457)
    );
    let dur = Instant::now() - start;
    println!("{:?}", dur);
  }
}
