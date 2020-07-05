use crate::model::mechanics::Mechanics;
use crate::model::pokemon::Level;

lazy_static! {
  static ref iv_combinations: Vec<(i32, i32, i32)> = {
    (0..=15).into_iter()
      .flat_map(|i|
        (0..=15).into_iter()
          .flat_map(|j|
            (0..15).into_iter()
              .map(|k| (i, j, k))))
              .collect()
  };
}

fn optimize_iv(pok: &str) -> Result<(), ()> {
  let mech = Mechanics::instance();
  let min_level = Level { level: 1, a_half: false };



  Ok(())
}
