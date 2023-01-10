//! VCF header record and components.

pub mod key;
pub(crate) mod parser;
pub mod value;

pub use self::key::Key;

use std::{error, fmt, str::FromStr};

use self::value::{
    map::{self, AlternativeAllele, Contig, Filter, Format, Info, Meta, Other},
    Map,
};
use super::{file_format, FileFormat};

pub(crate) const PREFIX: &str = "##";

/// A VCF header record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Record {
    /// An `ALT` record.
    AlternativeAllele(
        crate::record::alternate_bases::allele::Symbol,
        Map<AlternativeAllele>,
    ),
    /// An `assembly` record.
    Assembly(String),
    /// A `contig` record.
    Contig(map::contig::Name, Map<Contig>),
    /// A `fileformat` record.
    FileFormat(FileFormat),
    /// A `FILTER` record.
    Filter(String, Map<Filter>),
    /// A `FORMAT` record.
    Format(crate::header::format::Key, Map<Format>),
    /// An `INFO` record.
    Info(crate::header::info::Key, Map<Info>),
    /// A `META` record.
    Meta(String, Map<Meta>),
    /// A `pedigreeDB` record.
    PedigreeDb(String),
    /// A nonstadard record.
    Other(Key, value::Other),
}

/// An error returned when a raw VCF header record fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is invalid.
    Invalid,
    /// The file format record is invalid.
    InvalidFileFormat(file_format::ParseError),
    /// An INFO record is invalid.
    InvalidInfo(map::TryFromFieldsError),
    /// A FILTER record is invalid.
    InvalidFilter(map::TryFromFieldsError),
    /// A FORMAT record is invalid.
    InvalidFormat(map::TryFromFieldsError),
    /// An ALT record is invalid.
    InvalidAlternativeAllele(map::TryFromFieldsError),
    /// A contig record is invalid.
    InvalidContig(map::TryFromFieldsError),
    /// A META record is invalid.
    InvalidMeta(map::TryFromFieldsError),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Invalid => None,
            Self::InvalidFileFormat(e) => Some(e),
            Self::InvalidInfo(e)
            | Self::InvalidFilter(e)
            | Self::InvalidFormat(e)
            | Self::InvalidAlternativeAllele(e)
            | Self::InvalidContig(e)
            | Self::InvalidMeta(e) => Some(e),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid => f.write_str("invalid input"),
            Self::InvalidFileFormat(_) => write!(f, "invalid {}", key::FILE_FORMAT),
            Self::InvalidInfo(_) => write!(f, "invalid {}", key::INFO),
            Self::InvalidFilter(_) => write!(f, "invalid {}", key::FILTER),
            Self::InvalidFormat(_) => write!(f, "invalid {}", key::FORMAT),
            Self::InvalidAlternativeAllele(_) => write!(f, "invalid {}", key::ALTERNATIVE_ALLELE),
            Self::InvalidContig(_) => write!(f, "invalid {}", key::CONTIG),
            Self::InvalidMeta(_) => write!(f, "invalid {}", key::META),
        }
    }
}

impl FromStr for Record {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from((FileFormat::default(), s))
    }
}

impl TryFrom<(FileFormat, &str)> for Record {
    type Error = ParseError;

    fn try_from((file_format, s): (FileFormat, &str)) -> Result<Self, Self::Error> {
        use self::parser::Value;

        let (_, (raw_key, value)) = parser::parse(s).map_err(|_| ParseError::Invalid)?;

        match Key::from(raw_key) {
            key::FILE_FORMAT => match value {
                Value::String(s) => {
                    let file_format = s.parse().map_err(ParseError::InvalidFileFormat)?;
                    Ok(Self::FileFormat(file_format))
                }
                _ => Err(ParseError::Invalid),
            },
            key::INFO => match value {
                Value::Struct(fields) => {
                    let id = get_field(&fields, "ID")
                        .ok_or(ParseError::Invalid)
                        .and_then(|id| id.parse().map_err(|_| ParseError::Invalid))?;

                    let info = Map::<Info>::try_from((file_format, fields))
                        .map_err(ParseError::InvalidInfo)?;

                    if file_format >= FileFormat::new(4, 3)
                        && !matches!(id, super::info::Key::Other(_))
                    {
                        validate_info_type_fields(&id, info.number(), info.ty())?;
                    }

                    Ok(Self::Info(id, info))
                }
                _ => Err(ParseError::Invalid),
            },
            key::FILTER => match value {
                Value::Struct(fields) => {
                    let id = get_field(&fields, "ID")
                        .map(|v| v.into())
                        .ok_or(ParseError::Invalid)?;

                    let filter =
                        Map::<Filter>::try_from(fields).map_err(|_| ParseError::Invalid)?;

                    Ok(Self::Filter(id, filter))
                }
                _ => Err(ParseError::Invalid),
            },
            key::FORMAT => match value {
                Value::Struct(fields) => {
                    let id = get_field(&fields, "ID")
                        .ok_or(ParseError::Invalid)
                        .and_then(|id| id.parse().map_err(|_| ParseError::Invalid))?;

                    let format = Map::<Format>::try_from((file_format, fields))
                        .map_err(|_| ParseError::Invalid)?;

                    if file_format >= FileFormat::new(4, 3)
                        && !matches!(id, super::format::Key::Other(_))
                    {
                        validate_format_type_fields(&id, format.number(), format.ty())?;
                    }

                    Ok(Self::Format(id, format))
                }
                _ => Err(ParseError::Invalid),
            },
            key::ALTERNATIVE_ALLELE => match value {
                Value::Struct(fields) => {
                    let id = get_field(&fields, "ID")
                        .ok_or(ParseError::Invalid)
                        .and_then(|id| id.parse().map_err(|_| ParseError::Invalid))?;

                    let alternative_allele = Map::<AlternativeAllele>::try_from(fields)
                        .map_err(|_| ParseError::Invalid)?;

                    Ok(Self::AlternativeAllele(id, alternative_allele))
                }
                _ => Err(ParseError::Invalid),
            },
            key::ASSEMBLY => match value {
                Value::String(s) => Ok(Self::Assembly(s)),
                _ => Err(ParseError::Invalid),
            },
            key::CONTIG => match value {
                Value::Struct(fields) => {
                    let id = get_field(&fields, "ID")
                        .ok_or(ParseError::Invalid)
                        .and_then(|id| id.parse().map_err(|_| ParseError::Invalid))?;

                    let contig =
                        Map::<Contig>::try_from(fields).map_err(|_| ParseError::Invalid)?;

                    Ok(Self::Contig(id, contig))
                }
                _ => Err(ParseError::Invalid),
            },
            key::META => match value {
                Value::Struct(fields) => {
                    let id = get_field(&fields, "ID")
                        .map(|v| v.into())
                        .ok_or(ParseError::Invalid)?;

                    let meta = Map::<Meta>::try_from(fields).map_err(|_| ParseError::Invalid)?;

                    Ok(Self::Meta(id, meta))
                }
                _ => Err(ParseError::Invalid),
            },
            key::PEDIGREE_DB => match value {
                Value::String(s) => Ok(Self::PedigreeDb(s)),
                _ => Err(ParseError::Invalid),
            },
            k => {
                let v = match value {
                    Value::String(s) => value::Other::from(s),
                    Value::Struct(fields) => {
                        let id = get_field(&fields, "ID")
                            .map(|v| v.into())
                            .ok_or(ParseError::Invalid)?;

                        let map =
                            Map::<Other>::try_from(fields).map_err(|_| ParseError::Invalid)?;

                        value::Other::from((id, map))
                    }
                };

                Ok(Self::Other(k, v))
            }
        }
    }
}

fn get_field<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

fn validate_format_type_fields(
    id: &super::format::Key,
    actual_number: super::Number,
    actual_type: super::format::Type,
) -> Result<(), ParseError> {
    use crate::header::format::key;

    let expected_number = key::number(id).unwrap();

    if actual_number != expected_number {
        return Err(ParseError::Invalid);
    }

    let expected_type = key::ty(id).unwrap();

    if actual_type != expected_type {
        return Err(ParseError::Invalid);
    }

    Ok(())
}

fn validate_info_type_fields(
    id: &super::info::Key,
    actual_number: super::Number,
    actual_type: super::info::Type,
) -> Result<(), ParseError> {
    use super::info::key;

    let expected_number = key::number(id).unwrap();

    if actual_number != expected_number {
        return Err(ParseError::Invalid);
    }

    let expected_type = key::ty(id).unwrap();

    if actual_type != expected_type {
        return Err(ParseError::Invalid);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() -> Result<(), ParseError> {
        let line = "##fileformat=VCFv4.3";
        assert_eq!(line.parse(), Ok(Record::FileFormat(FileFormat::new(4, 3))));

        let line =
            r#"##INFO=<ID=NS,Number=1,Type=Integer,Description="Number of samples with data">"#;
        assert!(matches!(line.parse(), Ok(Record::Info(..))));

        assert!("".parse::<Record>().is_err());

        Ok(())
    }
}
