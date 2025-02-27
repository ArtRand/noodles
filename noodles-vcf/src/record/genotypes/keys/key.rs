//! VCF record genotypes key.

use std::{borrow::Borrow, error, fmt, str::FromStr};

/// Read depth for each allele (`AD`).
pub const READ_DEPTHS: Key = Key::Standard(Standard::ReadDepths);

/// Read depth for each allele on the forward strand (`ADF`).
pub const FORWARD_STRAND_READ_DEPTHS: Key = Key::Standard(Standard::ForwardStrandReadDepths);

/// Read depth for each allele on the reverse strand (`ADR`).
pub const REVERSE_STRAND_READ_DEPTHS: Key = Key::Standard(Standard::ReverseStrandReadDepths);

/// Read depth (`DP`).
pub const READ_DEPTH: Key = Key::Standard(Standard::ReadDepth);

/// Expected alternate allele counts (`EC`).
pub const EXPECTED_ALTERNATE_ALLELE_COUNTS: Key =
    Key::Standard(Standard::ExpectedAlternateAlleleCounts);

/// Filter indicating if this genotype was "called" (`FT`).
pub const FILTER: Key = Key::Standard(Standard::Filter);

/// Genotype likelihoods (`GL`).
pub const GENOTYPE_LIKELIHOODS: Key = Key::Standard(Standard::GenotypeLikelihoods);

/// Genotype posterior probabilities (`GP`).
pub const GENOTYPE_POSTERIOR_PROBABILITIES: Key =
    Key::Standard(Standard::GenotypePosteriorProbabilities);

/// Conditional genotype quality (`GQ`).
pub const CONDITIONAL_GENOTYPE_QUALITY: Key = Key::Standard(Standard::ConditionalGenotypeQuality);

/// Genotype (`GT`).
pub const GENOTYPE: Key = Key::Standard(Standard::Genotype);

/// Haplotype quality (`HQ`).
pub const HAPLOTYPE_QUALITY: Key = Key::Standard(Standard::HaplotypeQuality);

/// RMS mapping quality (`MQ`).
pub const MAPPING_QUALITY: Key = Key::Standard(Standard::MappingQuality);

/// Phred-scaled genotype likelihoods rounded to the closest integer (`PL`).
pub const ROUNDED_GENOTYPE_LIKELIHOODS: Key = Key::Standard(Standard::RoundedGenotypeLikelihoods);

/// Phred-scaled genotype posterior probabilities rounded to the closest integer (`PP`).
pub const ROUNDED_GENOTYPE_POSTERIOR_PROBABILITIES: Key =
    Key::Standard(Standard::RoundedGenotypePosteriorProbabilities);

/// Phasing quality (`PQ`).
pub const PHASING_QUALITY: Key = Key::Standard(Standard::PhasingQuality);

/// Phase set (`PS`).
pub const PHASE_SET: Key = Key::Standard(Standard::PhaseSet);

/// Phase set list (`PSL`).
///
/// Added in VCF 4.4.
pub const PHASE_SET_LIST: Key = Key::Standard(Standard::PhaseSetList);

/// Phase set list ordinal (`PSO`).
///
/// Added in VCF 4.4.
pub const PHASE_SET_LIST_ORDINALS: Key = Key::Standard(Standard::PhaseSetListOrdinals);

/// Phase set list quality (`PSQ`).
///
/// Added in VCF 4.4.
pub const PHASE_SET_LIST_QUALITIES: Key = Key::Standard(Standard::PhaseSetListQualities);

/// Copy number genotype for imprecise events (`CN`).
pub const GENOTYPE_COPY_NUMBER: Key = Key::Standard(Standard::GenotypeCopyNumber);

/// Confidence interval around copy number (`CICN`).
///
/// Added in VCF 4.4.
pub const COPY_NUMBER_CONFIDENCE_INTERVAL: Key =
    Key::Standard(Standard::CopyNumberConfidenceInterval);

/// Copy number genotype quality for imprecise events (`CNQ`).
pub const GENOTYPE_COPY_NUMBER_QUALITY: Key = Key::Standard(Standard::GenotypeCopyNumberQuality);

/// Copy number genotype likelihood for imprecise events (`CNL`).
pub const GENOTYPE_COPY_NUMBER_LIKELIHOODS: Key =
    Key::Standard(Standard::GenotypeCopyNumberLikelihoods);

/// Copy number posterior probabilities (`CNP`).
pub const GENOTYPE_COPY_NUMBER_POSTERIOR_PROBABILITIES: Key =
    Key::Standard(Standard::GenotypeCopyNumberPosteriorProbabilities);

/// Phred style probability score that the variant is novel (`NQ`).
pub const NOVEL_VARIANT_QUALITY_SCORE: Key = Key::Standard(Standard::NovelVariantQualityScore);

/// Unique haplotype identifier (`HAP`).
pub const HAPLOTYPE_ID: Key = Key::Standard(Standard::HaplotypeId);

/// Unique identifier of ancestral haplotype (`AHAP`).
pub const ANCESTRAL_HAPLOTYPE_ID: Key = Key::Standard(Standard::AncestralHaplotypeId);

/// A reserved VCF header format key.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Standard {
    /// Read depth for each allele (`AD`).
    ReadDepths,
    /// Read depth for each allele on the forward strand (`ADF`).
    ForwardStrandReadDepths,
    /// Read depth for each allele on the reverse strand (`ADR`).
    ReverseStrandReadDepths,
    /// Read depth (`DP`).
    ReadDepth,
    /// Expected alternate allele counts (`EC`).
    ExpectedAlternateAlleleCounts,
    /// Filter indicating if this genotype was "called" (`FT`).
    Filter,
    /// Genotype likelihoods (`GL`).
    GenotypeLikelihoods,
    /// Genotype posterior probabilities (`GP`).
    GenotypePosteriorProbabilities,
    /// Conditional genotype quality (`GQ`).
    ConditionalGenotypeQuality,
    /// Genotype (`GT`).
    Genotype,
    /// Haplotype quality (`HQ`).
    HaplotypeQuality,
    /// RMS mapping quality (`MQ`).
    MappingQuality,
    /// Phred-scaled genotype likelihoods rounded to the closest integer (`PL`).
    RoundedGenotypeLikelihoods,
    /// Phred-scaled genotype posterior probabilities rounded to the closest integer (`PP`).
    RoundedGenotypePosteriorProbabilities,
    /// Phasing quality (`PQ`).
    PhasingQuality,
    /// Phase set (`PS`).
    PhaseSet,
    /// Phase set list (`PSL`).
    ///
    /// Added in VCF 4.4.
    PhaseSetList,
    /// Phase set list ordinal (`PSO`).
    ///
    /// Added in VCF 4.4.
    PhaseSetListOrdinals,
    /// Phase set list quality (`PSQ`).
    ///
    /// Added in VCF 4.4.
    PhaseSetListQualities,

    /// Copy number genotype for imprecise events (`CN`).
    GenotypeCopyNumber,
    /// Confidence interval around copy number (`CICN`).
    ///
    /// Added in VCF 4.4.
    CopyNumberConfidenceInterval,
    /// Copy number genotype quality for imprecise events (`CNQ`).
    GenotypeCopyNumberQuality,
    /// Copy number genotype likelihood for imprecise events (`CNL`).
    GenotypeCopyNumberLikelihoods,
    /// Copy number posterior probabilities (`CNP`).
    GenotypeCopyNumberPosteriorProbabilities,
    /// Phred style probability score that the variant is novel (`NQ`).
    NovelVariantQualityScore,
    /// Unique haplotype identifier (`HAP`).
    HaplotypeId,
    /// Unique identifier of ancestral haplotype (`AHAP`).
    AncestralHaplotypeId,
}

impl AsRef<str> for Standard {
    fn as_ref(&self) -> &str {
        match self {
            Self::ReadDepths => "AD",
            Self::ForwardStrandReadDepths => "ADF",
            Self::ReverseStrandReadDepths => "ADR",
            Self::ReadDepth => "DP",
            Self::ExpectedAlternateAlleleCounts => "EC",
            Self::Filter => "FT",
            Self::GenotypeLikelihoods => "GL",
            Self::GenotypePosteriorProbabilities => "GP",
            Self::ConditionalGenotypeQuality => "GQ",
            Self::Genotype => "GT",
            Self::HaplotypeQuality => "HQ",
            Self::MappingQuality => "MQ",
            Self::RoundedGenotypeLikelihoods => "PL",
            Self::RoundedGenotypePosteriorProbabilities => "PP",
            Self::PhasingQuality => "PQ",
            Self::PhaseSet => "PS",
            Self::PhaseSetList => "PSL",
            Self::PhaseSetListOrdinals => "PSO",
            Self::PhaseSetListQualities => "PSQ",

            Self::GenotypeCopyNumber => "CN",
            Self::CopyNumberConfidenceInterval => "CICN",
            Self::GenotypeCopyNumberQuality => "CNQ",
            Self::GenotypeCopyNumberLikelihoods => "CNL",
            Self::GenotypeCopyNumberPosteriorProbabilities => "CNP",
            Self::NovelVariantQualityScore => "NQ",
            Self::HaplotypeId => "HAP",
            Self::AncestralHaplotypeId => "AHAP",
        }
    }
}

/// An error returned when a raw VCF record genotype field key fails to parse.
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
            Self::Empty => f.write_str("empty input"),
            Self::Invalid => f.write_str("invalid input"),
        }
    }
}

impl FromStr for Standard {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AD" => Ok(Self::ReadDepths),
            "ADF" => Ok(Self::ForwardStrandReadDepths),
            "ADR" => Ok(Self::ReverseStrandReadDepths),
            "DP" => Ok(Self::ReadDepth),
            "EC" => Ok(Self::ExpectedAlternateAlleleCounts),
            "FT" => Ok(Self::Filter),
            "GL" => Ok(Self::GenotypeLikelihoods),
            "GP" => Ok(Self::GenotypePosteriorProbabilities),
            "GQ" => Ok(Self::ConditionalGenotypeQuality),
            "GT" => Ok(Self::Genotype),
            "HQ" => Ok(Self::HaplotypeQuality),
            "MQ" => Ok(Self::MappingQuality),
            "PL" => Ok(Self::RoundedGenotypeLikelihoods),
            "PP" => Ok(Self::RoundedGenotypePosteriorProbabilities),
            "PQ" => Ok(Self::PhasingQuality),
            "PS" => Ok(Self::PhaseSet),
            "PSL" => Ok(Self::PhaseSetList),
            "PSO" => Ok(Self::PhaseSetListOrdinals),
            "PSQ" => Ok(Self::PhaseSetListQualities),

            "CN" => Ok(Self::GenotypeCopyNumber),
            "CICN" => Ok(Self::CopyNumberConfidenceInterval),
            "CNQ" => Ok(Self::GenotypeCopyNumberQuality),
            "CNL" => Ok(Self::GenotypeCopyNumberLikelihoods),
            "CNP" => Ok(Self::GenotypeCopyNumberPosteriorProbabilities),
            "NQ" => Ok(Self::NovelVariantQualityScore),
            "HAP" => Ok(Self::HaplotypeId),
            "AHAP" => Ok(Self::AncestralHaplotypeId),

            _ => Err(ParseError::Invalid),
        }
    }
}

/// A non-reserved VCF header format key.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Other(String);

impl AsRef<str> for Other {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Other {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl FromStr for Other {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if is_valid_name(s) {
            Ok(Self(s.into()))
        } else {
            Err(ParseError::Invalid)
        }
    }
}

// § 1.6.2 Genotype fields
fn is_valid_name_char(c: char) -> bool {
    matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '.')
}

fn is_valid_name(s: &str) -> bool {
    let mut chars = s.chars();

    if let Some(c) = chars.next() {
        if !matches!(c, 'A'..='Z' | 'a'..='z' | '_') {
            return false;
        }
    }

    chars.all(is_valid_name_char)
}

/// A VCF header format key.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Key {
    /// A reserved key.
    Standard(Standard),
    /// Any other non-reserved key.
    Other(Other),
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        match self {
            Self::Standard(k) => k.as_ref(),
            Self::Other(k) => k.as_ref(),
        }
    }
}

impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for Key {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        s.parse()
            .map(Self::Standard)
            .or_else(|_| s.parse().map(Self::Other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        assert_eq!(READ_DEPTHS.to_string(), "AD");
        assert_eq!(FORWARD_STRAND_READ_DEPTHS.to_string(), "ADF");
        assert_eq!(REVERSE_STRAND_READ_DEPTHS.to_string(), "ADR");
        assert_eq!(READ_DEPTH.to_string(), "DP");
        assert_eq!(EXPECTED_ALTERNATE_ALLELE_COUNTS.to_string(), "EC");
        assert_eq!(FILTER.to_string(), "FT");
        assert_eq!(GENOTYPE_LIKELIHOODS.to_string(), "GL");
        assert_eq!(GENOTYPE_POSTERIOR_PROBABILITIES.to_string(), "GP");
        assert_eq!(CONDITIONAL_GENOTYPE_QUALITY.to_string(), "GQ");
        assert_eq!(GENOTYPE.to_string(), "GT");
        assert_eq!(HAPLOTYPE_QUALITY.to_string(), "HQ");
        assert_eq!(MAPPING_QUALITY.to_string(), "MQ");
        assert_eq!(ROUNDED_GENOTYPE_LIKELIHOODS.to_string(), "PL");
        assert_eq!(ROUNDED_GENOTYPE_POSTERIOR_PROBABILITIES.to_string(), "PP");
        assert_eq!(PHASING_QUALITY.to_string(), "PQ");
        assert_eq!(PHASE_SET.to_string(), "PS");
        assert_eq!(PHASE_SET_LIST.to_string(), "PSL");
        assert_eq!(PHASE_SET_LIST_ORDINALS.to_string(), "PSO");
        assert_eq!(PHASE_SET_LIST_QUALITIES.to_string(), "PSQ");

        assert_eq!(GENOTYPE_COPY_NUMBER.to_string(), "CN");
        assert_eq!(COPY_NUMBER_CONFIDENCE_INTERVAL.to_string(), "CICN");
        assert_eq!(GENOTYPE_COPY_NUMBER_QUALITY.to_string(), "CNQ");
        assert_eq!(GENOTYPE_COPY_NUMBER_LIKELIHOODS.to_string(), "CNL");
        assert_eq!(
            GENOTYPE_COPY_NUMBER_POSTERIOR_PROBABILITIES.to_string(),
            "CNP"
        );
        assert_eq!(NOVEL_VARIANT_QUALITY_SCORE.to_string(), "NQ");
        assert_eq!(HAPLOTYPE_ID.to_string(), "HAP");
        assert_eq!(ANCESTRAL_HAPLOTYPE_ID.to_string(), "AHAP");

        assert_eq!(Key::Other(Other(String::from("NDLS"))).to_string(), "NDLS");
    }

    #[test]
    fn test_from_str() {
        assert_eq!("AD".parse(), Ok(READ_DEPTHS));
        assert_eq!("ADF".parse(), Ok(FORWARD_STRAND_READ_DEPTHS));
        assert_eq!("ADR".parse(), Ok(REVERSE_STRAND_READ_DEPTHS));
        assert_eq!("DP".parse(), Ok(READ_DEPTH));
        assert_eq!("EC".parse(), Ok(EXPECTED_ALTERNATE_ALLELE_COUNTS));
        assert_eq!("FT".parse(), Ok(FILTER));
        assert_eq!("GL".parse(), Ok(GENOTYPE_LIKELIHOODS));
        assert_eq!("GP".parse(), Ok(GENOTYPE_POSTERIOR_PROBABILITIES));
        assert_eq!("GQ".parse(), Ok(CONDITIONAL_GENOTYPE_QUALITY));
        assert_eq!("GT".parse(), Ok(GENOTYPE));
        assert_eq!("HQ".parse(), Ok(HAPLOTYPE_QUALITY));
        assert_eq!("MQ".parse(), Ok(MAPPING_QUALITY));
        assert_eq!("PL".parse(), Ok(ROUNDED_GENOTYPE_LIKELIHOODS));
        assert_eq!("PP".parse(), Ok(ROUNDED_GENOTYPE_POSTERIOR_PROBABILITIES));
        assert_eq!("PQ".parse(), Ok(PHASING_QUALITY));
        assert_eq!("PS".parse(), Ok(PHASE_SET));
        assert_eq!("PSL".parse(), Ok(PHASE_SET_LIST));
        assert_eq!("PSO".parse(), Ok(PHASE_SET_LIST_ORDINALS));
        assert_eq!("PSQ".parse(), Ok(PHASE_SET_LIST_QUALITIES));

        assert_eq!("CN".parse(), Ok(GENOTYPE_COPY_NUMBER));
        assert_eq!("CICN".parse(), Ok(COPY_NUMBER_CONFIDENCE_INTERVAL));
        assert_eq!("CNQ".parse(), Ok(GENOTYPE_COPY_NUMBER_QUALITY));
        assert_eq!("CNL".parse(), Ok(GENOTYPE_COPY_NUMBER_LIKELIHOODS));
        assert_eq!(
            "CNP".parse(),
            Ok(GENOTYPE_COPY_NUMBER_POSTERIOR_PROBABILITIES)
        );
        assert_eq!("NQ".parse(), Ok(NOVEL_VARIANT_QUALITY_SCORE));
        assert_eq!("HAP".parse(), Ok(HAPLOTYPE_ID));
        assert_eq!("AHAP".parse(), Ok(ANCESTRAL_HAPLOTYPE_ID));

        assert_eq!("NDLS".parse(), Ok(Key::Other(Other(String::from("NDLS")))));

        assert_eq!("".parse::<Key>(), Err(ParseError::Empty));
        assert_eq!("8D".parse::<Key>(), Err(ParseError::Invalid));
        assert_eq!(".N".parse::<Key>(), Err(ParseError::Invalid));
        assert_eq!("A!".parse::<Key>(), Err(ParseError::Invalid));
    }
}
