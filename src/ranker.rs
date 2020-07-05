use crate::model::{Level, Mechanics, PokemonInstance};

fn max_statproduct(mech: &Mechanics, pokemon_id: &str, cap: usize) -> Result<(), ()> {
  let combs = all_iv_combs();
  let pok = mech.pokemon(pokemon_id).ok_or_else(|| ())?;
  let fast_move = &pok.fast_moves[0];
  let charged_move = &pok.charged_moves[0];
  const MIN_LEVEL: Level = Level { level: 1, a_half: false };

  use std::time::{Instant, Duration};
  let mut times: Vec<Duration> = Vec::with_capacity(16 * 16 * 16 * 80);

  let instance = combs.into_iter()
    .map(|(atk, def, sta)| {
      let mut level = Level {
        level: 40,
        a_half: false,
      };

      while level > MIN_LEVEL {

        let cpm = mech.cp_multiplier(&level);

        let start = Instant::now();
        let a = (pok.stats.base_attack + atk) as f64;
        let d = (pok.stats.base_defense + def) as f64;
        let s = (pok.stats.base_stamina + sta) as f64;

        let cp = f64::floor(a * d.sqrt() * s.sqrt() * cpm * cpm / 10.) as u32;
        let stat_product = a * d * s as f64;
        times.push(Instant::now() - start);

        if cp <= cap as _ {
          return (atk, def, sta, level, stat_product)
        }
        level = level.prev();
      }
      /*while level > MIN_LEVEL {
        // UNWRAP OK: 
        // - pokemon exists because pok exists,
        // - fast move exists because every pokemon has at least one,
        // - charged move exists for the same reason
        // - no other way of failing
        let cpm = mech.cp_multiplier(&level);

        let start = Instant::now();
        let pok_inst = PokemonInstance::new(
          pok.clone(), level, cpm,
          atk, def, sta,
          fast_move.clone(),
          charged_move.clone(),
          charged_move.clone(),
        );
        times.push(Instant::now() - start);

        if pok_inst.cp() <= cap as _ {
          return (atk, def, sta, level, pok_inst.stat_product())
        
        }
        level = level.prev();
      }*/

      (atk, def, sta, level, 0.) // should be unreachable
    })
    .fold((0, 0, 0, MIN_LEVEL, 0.), |max, cur| {
      if max.4 > cur.4 {
        max
      } else {
        cur
      }
    });

  println!("{:?}", instance);
  let now = Instant::now();
  let count = times.len();
  let mean = times.into_iter().fold(now - now, |a, b| a + b) / (count as _);
  println!("{:?} {}", mean, count);

  Ok(())
}

fn all_iv_combs() -> Vec<(u16, u16, u16)> {
  (0..=15).into_iter().flat_map(move |i|
    (0..=15).into_iter().flat_map(move |j|
      (0..=15).into_iter().map(move |k| (i, j, k))))
      .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::gamemaster::GameMaster;

  use std::convert::TryFrom;

  #[test]
  fn test_iv_combs() {
    use std::time::Instant;

    let start = Instant::now();
    let iv_combs = super::all_iv_combs();
    let dur = Instant::now() - start;

    assert_eq!(iv_combs.len(), 16*16*16);
    println!("{:?}", dur);
  }

  #[test]
  fn test_max_statproduct() {
    let gms = std::fs::read_to_string("data/gamemaster.json").unwrap();
    let gm = serde_json::from_str::<GameMaster>(&gms).unwrap();

    let mech = Mechanics::try_from(gm).unwrap();

    use std::time::Instant;

    let start = Instant::now();
    max_statproduct(&mech, "ALTARIA", 1500);
    let dur = Instant::now() - start;
    println!("{:?}", dur);
  }
}
