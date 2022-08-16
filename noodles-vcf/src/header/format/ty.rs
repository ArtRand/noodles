//! VCF header genotype format field value type.

use std::{error, fmt, str::FromStr};

/// A VCF header genotype format field value type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    /// A 32-bit integer.
    Integer,
    /// A single-precision floating-point.
    Float,
    /// A character.
    Character,
    /// A string.
    String,
}

impl AsRef<str> for Type {
    fn as_ref(&self) -> &str {
        match self {
            Self::Integer => "Integer",
            Self::Float => "Float",
            Self::Character => "Character",
            Self::String => "String",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

/// An error returned when a VCF header genotype format field type fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError(String);

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid format type: expected {{Integer, Float, Character, String}}, got {}",
            self.0
        )
    }
}

impl FromStr for Type {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Integer" => Ok(Self::Integer),
            "Float" => Ok(Self::Float),
            "Character" => Ok(Self::Character),
            "String" => Ok(Self::String),
            _ => Err(ParseError(s.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        assert_eq!(Type::Integer.to_string(), "Integer");
        assert_eq!(Type::Float.to_string(), "Float");
        assert_eq!(Type::Character.to_string(), "Character");
        assert_eq!(Type::String.to_string(), "String");
    }

    #[test]
    fn test_from_str() -> Result<(), ParseError> {
        assert_eq!("Integer".parse::<Type>()?, Type::Integer);
        assert_eq!("Float".parse::<Type>()?, Type::Float);
        assert_eq!("Character".parse::<Type>()?, Type::Character);
        assert_eq!("String".parse::<Type>()?, Type::String);

        assert!("".parse::<Type>().is_err());
        assert!("Noodles".parse::<Type>().is_err());

        Ok(())
    }
}
