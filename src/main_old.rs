#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fs;

mod error;
mod gamemaster;
mod model;

#[derive(Debug)]
enum Error {
  None,
  Error(String),
}

impl From<std::num::ParseIntError> for Error {
  fn from(e: std::num::ParseIntError) -> Error {
    Error::Error(format!("Couldn't parse int: {}", e))
  }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum MatchOutcome {
  Win(u16),
  Lose(u16),
  Tie,
}

impl From<u16> for MatchOutcome {
  fn from(i: u16) -> MatchOutcome {
    match i {
      500u16 => MatchOutcome::Tie,
      0..=499u16 => MatchOutcome::Lose(500 - i),
      501u16..=std::u16::MAX => MatchOutcome::Win(i - 500),
    }
  }
}

#[derive(Debug)]
struct Matchup<'a> {
  pokemon: &'a str,
  results: Vec<MatchOutcome>,
}

impl<'a> TryFrom<&'a str> for Matchup<'a> {
  type Error = Error;

  fn try_from(line: &'a str) -> Result<Matchup<'a>> {
    let mut chunks = line.split(",");
    let pokemon = chunks.next().map_or_else(
      || Err(Error::Error(format!("Can't read Pokémon name field!"))),
      |i| Ok(i),
    )?;
    let results = chunks
      .into_iter()
      .map(|c| {
        Ok(MatchOutcome::from(
          c.parse::<u16>().map_err(|e| Error::from(e))?,
        ))
      })
      .collect::<Result<Vec<MatchOutcome>>>()?;

    Ok(Matchup { pokemon, results })
  }
}

#[derive(Debug)]
struct Team<'a> {
  team: [&'a Matchup<'a>; 6],
}

/*impl<'a> Team<'a> {
  fn ranking(&self) -> Vec<MatchOutcome> {

  }
}*/

fn into_enemies<'a>(line: &'a str) -> Vec<&'a str> {
  let mut m = line.split(",");
  m.next().unwrap_or_else(|| "");

  m.map(str::trim).collect()
}

fn main() -> Result<()> {
  let contents = fs::read_to_string("input.csv").expect("Couldn't read file");

  let mut lines = contents.split("\n");
  let enemies = into_enemies(
    lines
      .next()
      .map_or_else(|| Err(Error::Error("No enemies header".into())), |c| Ok(c))?,
  );

  println!("{:?}", enemies);

  for c in lines.into_iter().map(str::trim).map(Matchup::try_from) {
    println!("{:?}", c);
  }

  Ok(())
}
