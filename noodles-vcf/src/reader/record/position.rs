use std::{error, fmt, num};

use noodles_core as core;

use crate::record::Position;

/// An error when a raw VCF record position fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// The input is invalid.
    Invalid,
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty input"),
            Self::Invalid => write!(f, "invalid input"),
        }
    }
}

impl From<num::ParseIntError> for ParseError {
    fn from(e: num::ParseIntError) -> Self {
        match e.kind() {
            num::IntErrorKind::Empty => Self::Empty,
            _ => Self::Invalid,
        }
    }
}

impl From<ParseError> for core::Error {
    fn from(e: ParseError) -> Self {
        Self::new(core::error::Kind::Parse, e)
    }
}

pub(super) fn parse_position(s: &str) -> Result<Position, ParseError> {
    s.parse::<usize>().map(Position::from).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_position() {
        assert_eq!(parse_position("0"), Ok(Position::from(0)));
        assert_eq!(parse_position("8"), Ok(Position::from(8)));

        assert_eq!(parse_position(""), Err(ParseError::Empty));
        assert_eq!(parse_position("."), Err(ParseError::Invalid));
        assert_eq!(parse_position("ndls"), Err(ParseError::Invalid));
        assert_eq!(parse_position("-1"), Err(ParseError::Invalid));
    }
}
