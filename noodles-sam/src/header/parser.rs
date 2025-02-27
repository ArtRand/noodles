mod context;

use std::{collections::HashSet, error, fmt};

pub(crate) use self::context::Context;
use super::{
    record::{self, value::map::reference_sequence},
    Header, Record,
};

/// An error returned when a raw SAM header fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// A header record is not on the first line.
    UnexpectedHeader,
    /// The record is invalid.
    InvalidRecord(record::ParseError),
    /// A reference sequence name is duplicated.
    DuplicateReferenceSequenceName(reference_sequence::Name),
    /// A read group ID is duplicated.
    DuplicateReadGroupId(String),
    /// A program ID is duplicated.
    DuplicateProgramId(String),
    /// A comment record is invalid.
    InvalidComment,
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidRecord(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedHeader => f.write_str("unexpected @HD"),
            Self::InvalidRecord(_) => f.write_str("invalid record"),
            Self::DuplicateReferenceSequenceName(name) => {
                write!(f, "duplicate reference sequence name: {name}")
            }
            Self::DuplicateReadGroupId(id) => write!(f, "duplicate read group ID: {id}"),
            Self::DuplicateProgramId(id) => write!(f, "duplicate program ID: {id}"),
            Self::InvalidComment => f.write_str("invalid comment record"),
        }
    }
}

/// Parses a raw SAM header.
///
/// # Examples
///
/// ```
/// use noodles_sam as sam;
///
/// let s = "\
/// @HD\tVN:1.6\tSO:coordinate
/// @SQ\tSN:sq0\tLN:8
/// @SQ\tSN:sq1\tLN:13
/// ";
///
/// let header: sam::Header = s.parse()?;
///
/// assert!(header.header().is_some());
/// assert_eq!(header.reference_sequences().len(), 2);
/// assert!(header.read_groups().is_empty());
/// assert!(header.programs().is_empty());
/// assert!(header.comments().is_empty());
/// # Ok::<(), sam::header::ParseError>(())
/// ```
pub(super) fn parse(s: &str) -> Result<Header, ParseError> {
    let mut builder = Header::builder();

    let mut ctx = Context::default();

    let mut read_group_ids: HashSet<String> = HashSet::new();
    let mut reference_sequence_names: HashSet<reference_sequence::Name> = HashSet::new();
    let mut program_ids: HashSet<String> = HashSet::new();

    let mut lines = s.lines();

    if let Some(line) = lines.next() {
        let version = record::extract_version(line)
            .transpose()
            .map_err(ParseError::InvalidRecord)?;

        if let Some(version) = version {
            ctx = Context::from(version);
        }

        let record = Record::try_from((&ctx, line)).map_err(ParseError::InvalidRecord)?;

        builder = match record {
            Record::Header(header) => builder.set_header(header),
            Record::ReferenceSequence(name, reference_sequence) => {
                reference_sequence_names.insert(name.clone());
                builder.add_reference_sequence(name, reference_sequence)
            }
            Record::ReadGroup(id, read_group) => {
                read_group_ids.insert(id.clone());
                builder.add_read_group(id, read_group)
            }
            Record::Program(id, program) => {
                program_ids.insert(id.clone());
                builder.add_program(id, program)
            }
            Record::Comment(comment) => builder.add_comment(comment),
        };
    }

    for line in lines {
        let record = Record::try_from((&ctx, line)).map_err(ParseError::InvalidRecord)?;

        builder = match record {
            Record::Header(_) => return Err(ParseError::UnexpectedHeader),
            Record::ReferenceSequence(name, reference_sequence) => {
                if !reference_sequence_names.insert(name.clone()) {
                    return Err(ParseError::DuplicateReferenceSequenceName(name));
                }

                builder.add_reference_sequence(name, reference_sequence)
            }
            Record::ReadGroup(id, read_group) => {
                if !read_group_ids.insert(id.clone()) {
                    return Err(ParseError::DuplicateReadGroupId(id));
                }

                builder.add_read_group(id, read_group)
            }
            Record::Program(id, program) => {
                if !program_ids.insert(id.clone()) {
                    return Err(ParseError::DuplicateProgramId(id));
                }

                builder.add_program(id, program)
            }
            Record::Comment(comment) => builder.add_comment(comment),
        };
    }

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() -> Result<(), Box<dyn std::error::Error>> {
        use std::num::NonZeroUsize;

        use crate::header::record::value::map::{
            self,
            header::{SortOrder, Version},
            Map, Program, ReadGroup, ReferenceSequence,
        };

        let s = "\
@HD\tVN:1.6\tSO:coordinate
@SQ\tSN:sq0\tLN:8
@SQ\tSN:sq1\tLN:13
@RG\tID:rg0
@PG\tID:pg0\tPN:noodles
@CO\tndls
";

        let actual = parse(s)?;

        let expected = Header::builder()
            .set_header(
                Map::<map::Header>::builder()
                    .set_version(Version::new(1, 6))
                    .set_sort_order(SortOrder::Coordinate)
                    .build()?,
            )
            .add_reference_sequence(
                "sq0".parse()?,
                Map::<ReferenceSequence>::new(NonZeroUsize::try_from(8)?),
            )
            .add_reference_sequence(
                "sq1".parse()?,
                Map::<ReferenceSequence>::new(NonZeroUsize::try_from(13)?),
            )
            .add_read_group("rg0", Map::<ReadGroup>::default())
            .add_program(
                "pg0",
                Map::<Program>::builder().set_name("noodles").build()?,
            )
            .add_comment("ndls")
            .build();

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_parse_with_empty_input() -> Result<(), ParseError> {
        let header = parse("")?;

        assert!(header.header().is_none());
        assert!(header.reference_sequences().is_empty());
        assert!(header.read_groups().is_empty());
        assert!(header.programs().is_empty());
        assert!(header.comments().is_empty());

        Ok(())
    }

    #[test]
    fn test_parse_without_hd() -> Result<(), ParseError> {
        let header = parse("@SQ\tSN:sq0\tLN:8\n")?;
        assert!(header.header().is_none());
        assert_eq!(header.reference_sequences().len(), 1);
        Ok(())
    }

    #[test]
    fn test_parse_with_multiple_hd() {
        let s = "\
@HD\tVN:1.6\tSO:coordinate
@HD\tVN:1.6\tSO:coordinate
";

        assert_eq!(parse(s), Err(ParseError::UnexpectedHeader));
    }

    #[test]
    fn test_parse_with_duplicate_reference_sequence_names(
    ) -> Result<(), reference_sequence::name::ParseError> {
        let s = "\
@SQ\tSN:sq0\tLN:8
@SQ\tSN:sq0\tLN:8
";

        assert_eq!(
            parse(s),
            Err(ParseError::DuplicateReferenceSequenceName("sq0".parse()?))
        );

        Ok(())
    }

    #[test]
    fn test_parse_with_duplicate_read_group_ids() {
        let s = "\
@RG\tID:rg0
@RG\tID:rg0
";

        assert_eq!(
            parse(s),
            Err(ParseError::DuplicateReadGroupId(String::from("rg0")))
        );
    }

    #[test]
    fn test_parse_with_duplicate_program_ids() {
        let s = "\
@PG\tID:pg0
@PG\tID:pg0
";
        assert_eq!(
            parse(s),
            Err(ParseError::DuplicateProgramId(String::from("pg0")))
        );
    }
}
