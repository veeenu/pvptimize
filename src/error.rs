#[derive(Debug)]
pub enum Error {
  ParseError(String),
  BoundsError(String),
  ConversionError(String)
}