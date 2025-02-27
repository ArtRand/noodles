//! VCF header file format.

use std::{error, fmt, num, str::FromStr};

/// A VCF header file format.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FileFormat {
    major: u32,
    minor: u32,
}

static PREFIX: &str = "VCFv";

const MAJOR_VERSION: u32 = 4;
const MINOR_VERSION: u32 = 4;

const DELIMITER: char = '.';
const MAX_COMPONENT_COUNT: usize = 2;

impl FileFormat {
    /// Creates a file format.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::header::FileFormat;
    /// let file_format = FileFormat::new(4, 3);
    /// ```
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Returns the major version.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::header::FileFormat;
    /// let file_format = FileFormat::new(4, 3);
    /// assert_eq!(file_format.major(), 4);
    /// ```
    pub fn major(&self) -> u32 {
        self.major
    }

    /// Returns the minor version.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::header::FileFormat;
    /// let file_format = FileFormat::new(4, 3);
    /// assert_eq!(file_format.minor(), 3);
    /// ```
    pub fn minor(&self) -> u32 {
        self.minor
    }
}

impl Default for FileFormat {
    fn default() -> Self {
        Self {
            major: MAJOR_VERSION,
            minor: MINOR_VERSION,
        }
    }
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}", PREFIX, self.major(), DELIMITER, self.minor())
    }
}

/// An error returned when a raw VCF header file format fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// The prefix is invalid.
    InvalidPrefix,
    /// The major version is missing.
    MissingMajorVersion,
    /// The major version is invalid.
    InvalidMajorVersion(num::ParseIntError),
    /// The minor version is missing.
    MissingMinorVersion,
    /// The minor version is invalid.
    InvalidMinorVersion(num::ParseIntError),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidMajorVersion(e) | Self::InvalidMinorVersion(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty input"),
            Self::InvalidPrefix => f.write_str("invalid prefix"),
            Self::MissingMajorVersion => f.write_str("missing major version"),
            Self::InvalidMajorVersion(_) => f.write_str("invalid major version"),
            Self::MissingMinorVersion => f.write_str("missing minor version"),
            Self::InvalidMinorVersion(_) => f.write_str("invalid minor version"),
        }
    }
}

impl FromStr for FileFormat {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        let raw_version = match s.strip_prefix(PREFIX) {
            Some(t) => t,
            None => return Err(ParseError::InvalidPrefix),
        };

        if raw_version.is_empty() {
            return Err(ParseError::MissingMajorVersion);
        }

        let mut components = raw_version.splitn(MAX_COMPONENT_COUNT, DELIMITER);

        let major = components
            .next()
            .ok_or(ParseError::MissingMajorVersion)
            .and_then(|t| t.parse().map_err(ParseError::InvalidMajorVersion))?;

        let minor = components
            .next()
            .ok_or(ParseError::MissingMinorVersion)
            .and_then(|t| t.parse().map_err(ParseError::InvalidMinorVersion))?;

        Ok(Self::new(major, minor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let file_format = FileFormat::default();
        assert_eq!(file_format.major(), MAJOR_VERSION);
        assert_eq!(file_format.minor(), MINOR_VERSION);
    }

    #[test]
    fn test_fmt() {
        let file_format = FileFormat::new(4, 3);
        assert_eq!(file_format.to_string(), "VCFv4.3");
    }

    #[test]
    fn test_from_str() {
        assert_eq!("VCFv4.3".parse(), Ok(FileFormat::new(4, 3)));

        assert_eq!("".parse::<FileFormat>(), Err(ParseError::Empty));

        assert_eq!("4.3".parse::<FileFormat>(), Err(ParseError::InvalidPrefix));
        assert_eq!(
            "NDLv4.3".parse::<FileFormat>(),
            Err(ParseError::InvalidPrefix)
        );

        assert_eq!(
            "VCFv".parse::<FileFormat>(),
            Err(ParseError::MissingMajorVersion)
        );

        assert!(matches!(
            "VCFvx".parse::<FileFormat>(),
            Err(ParseError::InvalidMajorVersion(_))
        ));

        assert_eq!(
            "VCFv4".parse::<FileFormat>(),
            Err(ParseError::MissingMinorVersion)
        );

        assert!(matches!(
            "VCFv4.x".parse::<FileFormat>(),
            Err(ParseError::InvalidMinorVersion(_))
        ));
    }
}
