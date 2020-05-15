use std::{error, fmt, str::FromStr};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Kind {
    FileFormat,
    Info,
    Filter,
    Format,
    AlternativeAllele,
    Assembly,
    Contig,
    Other(String),
}

impl AsRef<str> for Kind {
    fn as_ref(&self) -> &str {
        match self {
            Self::FileFormat => "fileformat",
            Self::Info => "INFO",
            Self::Filter => "FILTER",
            Self::Format => "FORMAT",
            Self::AlternativeAllele => "ALT",
            Self::Assembly => "assembly",
            Self::Contig => "contig",
            Self::Other(s) => s,
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug)]
pub struct ParseError(String);

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid record kind: expected {{fileformat, INFO, FILTER, FORMAT}}, got {}",
            self.0
        )
    }
}

impl FromStr for Kind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Err(ParseError(s.into())),
            "fileformat" => Ok(Self::FileFormat),
            "INFO" => Ok(Self::Info),
            "FILTER" => Ok(Self::Filter),
            "FORMAT" => Ok(Self::Format),
            "ALT" => Ok(Self::AlternativeAllele),
            "assembly" => Ok(Self::Assembly),
            "contig" => Ok(Self::Contig),
            _ => Ok(Self::Other(s.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        assert_eq!(Kind::FileFormat.to_string(), "fileformat");
        assert_eq!(Kind::Info.to_string(), "INFO");
        assert_eq!(Kind::Filter.to_string(), "FILTER");
        assert_eq!(Kind::Format.to_string(), "FORMAT");
        assert_eq!(Kind::AlternativeAllele.to_string(), "ALT");
        assert_eq!(Kind::Assembly.to_string(), "assembly");
        assert_eq!(Kind::Contig.to_string(), "contig");
        assert_eq!(
            Kind::Other(String::from("fileDate")).to_string(),
            "fileDate"
        );
    }

    #[test]
    fn test_from_str() -> Result<(), ParseError> {
        assert_eq!("fileformat".parse::<Kind>()?, Kind::FileFormat);
        assert_eq!("INFO".parse::<Kind>()?, Kind::Info);
        assert_eq!("FILTER".parse::<Kind>()?, Kind::Filter);
        assert_eq!("FORMAT".parse::<Kind>()?, Kind::Format);
        assert_eq!("ALT".parse::<Kind>()?, Kind::AlternativeAllele);
        assert_eq!("assembly".parse::<Kind>()?, Kind::Assembly);
        assert_eq!("contig".parse::<Kind>()?, Kind::Contig);
        assert_eq!(
            "fileDate".parse::<Kind>()?,
            Kind::Other(String::from("fileDate"))
        );

        assert!("".parse::<Kind>().is_err());

        Ok(())
    }
}
