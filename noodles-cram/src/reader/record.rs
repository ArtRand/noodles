mod external_data_readers;

pub use external_data_readers::ExternalDataReaders;

use std::{error, fmt, io};

use bytes::Buf;
use noodles_bam as bam;
use noodles_core::Position;
use noodles_sam::{
    self as sam,
    record::{quality_scores::Score, sequence::Base},
    AlignmentRecord,
};

use super::num::get_itf8;
use crate::{
    container::ReferenceSequenceId,
    data_container::{
        compression_header::{
            data_series_encoding_map::DataSeries, encoding::Encoding,
            preservation_map::tag_ids_dictionary,
        },
        CompressionHeader,
    },
    huffman::CanonicalHuffmanDecoder,
    record::{
        feature::{self, substitution},
        Feature, Flags, NextMateFlags,
    },
    BitReader, Record,
};

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadRecordError {
    MissingDataSeriesEncoding(DataSeries),
    MissingTagEncoding(tag_ids_dictionary::Key),
    MissingExternalBlock(i32),
}

impl error::Error for ReadRecordError {}

impl fmt::Display for ReadRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDataSeriesEncoding(data_series) => {
                write!(f, "missing data series encoding: {:?}", data_series)
            }
            Self::MissingTagEncoding(key) => write!(f, "missing tag encoding: {:?}", key),
            Self::MissingExternalBlock(block_content_id) => {
                write!(f, "missing external block: {}", block_content_id)
            }
        }
    }
}

pub struct Reader<'a, CDR, EDR>
where
    CDR: Buf,
    EDR: Buf,
{
    compression_header: &'a CompressionHeader,
    core_data_reader: BitReader<CDR>,
    external_data_readers: ExternalDataReaders<EDR>,
    reference_sequence_id: ReferenceSequenceId,
    prev_alignment_start: Option<Position>,
}

impl<'a, CDR, EDR> Reader<'a, CDR, EDR>
where
    CDR: Buf,
    EDR: Buf,
{
    pub fn new(
        compression_header: &'a CompressionHeader,
        core_data_reader: BitReader<CDR>,
        external_data_readers: ExternalDataReaders<EDR>,
        reference_sequence_id: ReferenceSequenceId,
        initial_alignment_start: Option<Position>,
    ) -> Self {
        Self {
            compression_header,
            core_data_reader,
            external_data_readers,
            reference_sequence_id,
            prev_alignment_start: initial_alignment_start,
        }
    }

    pub fn read_record(&mut self) -> io::Result<Record> {
        let bam_bit_flags = self.read_bam_bit_flags()?;
        let cram_bit_flags = self.read_cram_bit_flags()?;

        let mut record = Record {
            bam_bit_flags,
            cram_bit_flags,
            ..Default::default()
        };

        let read_length = self.read_positional_data(&mut record)?;
        self.read_read_names(&mut record)?;
        self.read_mate_data(&mut record, bam_bit_flags, cram_bit_flags)?;

        record.tags = self.read_tag_data()?;

        if bam_bit_flags.is_unmapped() {
            self.read_unmapped_read(&mut record, cram_bit_flags, read_length)?;
        } else {
            self.read_mapped_read(&mut record, cram_bit_flags, read_length)?;
        }

        self.prev_alignment_start = record.alignment_start();

        Ok(record)
    }

    fn read_bam_bit_flags(&mut self) -> io::Result<sam::record::Flags> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .bam_bit_flags_encoding();

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| u16::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
        .map(sam::record::Flags::from)
    }

    fn read_cram_bit_flags(&mut self) -> io::Result<Flags> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .cram_bit_flags_encoding();

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| u8::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
        .map(Flags::from)
    }

    fn read_positional_data(&mut self, record: &mut Record) -> io::Result<usize> {
        use bam::record::reference_sequence_id::UNMAPPED;

        let reference_id = if self.reference_sequence_id.is_many() {
            self.read_reference_id()?
        } else {
            i32::from(self.reference_sequence_id)
        };

        record.reference_sequence_id = match reference_id {
            UNMAPPED => None,
            _ => usize::try_from(reference_id)
                .map(Some)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        };

        let read_length = self.read_read_length()?;
        record.read_length = read_length;

        record.alignment_start = self.read_alignment_start()?;
        record.read_group = self.read_read_group()?;

        Ok(read_length)
    }

    fn read_reference_id(&mut self) -> io::Result<i32> {
        self.compression_header
            .data_series_encoding_map()
            .reference_id_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::ReferenceId),
                )
            })
            .and_then(|encoding| {
                decode_itf8(
                    encoding,
                    &mut self.core_data_reader,
                    &mut self.external_data_readers,
                )
            })
    }

    fn read_read_length(&mut self) -> io::Result<usize> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .read_lengths_encoding();

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_alignment_start(&mut self) -> io::Result<Option<Position>> {
        let ap_data_series_delta = self
            .compression_header
            .preservation_map()
            .ap_data_series_delta();

        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .in_seq_positions_encoding();

        let alignment_start_or_delta = decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )?;

        let alignment_start = if ap_data_series_delta {
            let prev_alignment_start = i32::try_from(
                self.prev_alignment_start
                    .map(usize::from)
                    .unwrap_or_default(),
            )
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            prev_alignment_start + alignment_start_or_delta
        } else {
            alignment_start_or_delta
        };

        usize::try_from(alignment_start)
            .map(Position::new)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read_read_group(&mut self) -> io::Result<Option<usize>> {
        // § 10.2 "CRAM positional data" (2021-10-15): "-1 for no group".
        const MISSING: i32 = -1;

        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .read_groups_encoding();

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| match n {
            MISSING => Ok(None),
            _ => usize::try_from(n)
                .map(Some)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        })
    }

    fn read_read_names(&mut self, record: &mut Record) -> io::Result<()> {
        let preservation_map = self.compression_header.preservation_map();

        // Missing read names are generated when resolving mates.
        if preservation_map.read_names_included() {
            record.read_name = self.read_read_name()?;
        }

        Ok(())
    }

    fn read_read_name(&mut self) -> io::Result<Option<sam::record::ReadName>> {
        use sam::record::read_name::MISSING;

        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .read_names_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::ReadNames),
                )
            })?;

        let buf = decode_byte_array(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
            None,
        )?;

        match &buf[..] {
            MISSING => Ok(None),
            _ => sam::record::ReadName::try_from(buf)
                .map(Some)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    fn read_mate_data(
        &mut self,
        record: &mut Record,
        mut bam_flags: sam::record::Flags,
        flags: Flags,
    ) -> io::Result<()> {
        if flags.is_detached() {
            let next_mate_bit_flags = self.read_next_mate_bit_flags()?;
            record.next_mate_bit_flags = next_mate_bit_flags;

            if next_mate_bit_flags.is_on_negative_strand() {
                bam_flags |= sam::record::Flags::MATE_REVERSE_COMPLEMENTED;
            }

            if next_mate_bit_flags.is_unmapped() {
                bam_flags |= sam::record::Flags::MATE_UNMAPPED;
            }

            record.bam_bit_flags = bam_flags;

            let preservation_map = self.compression_header.preservation_map();

            if !preservation_map.read_names_included() {
                record.read_name = self.read_read_name()?;
            }

            record.next_fragment_reference_sequence_id =
                self.read_next_fragment_reference_sequence_id()?;

            record.next_mate_alignment_start = self.read_next_mate_alignment_start()?;
            record.template_size = self.read_template_size()?;
        } else if flags.has_mate_downstream() {
            record.distance_to_next_fragment = self.read_distance_to_next_fragment().map(Some)?;
        }

        Ok(())
    }

    fn read_next_mate_bit_flags(&mut self) -> io::Result<NextMateFlags> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .next_mate_bit_flags_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::NextMateBitFlags),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| u8::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
        .map(NextMateFlags::from)
    }

    fn read_next_fragment_reference_sequence_id(&mut self) -> io::Result<Option<usize>> {
        use bam::record::reference_sequence_id::UNMAPPED;

        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .next_fragment_reference_sequence_id_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(
                        DataSeries::NextFragmentReferenceSequenceId,
                    ),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|id| match id {
            UNMAPPED => Ok(None),
            _ => usize::try_from(id)
                .map(Some)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        })
    }

    fn read_next_mate_alignment_start(&mut self) -> io::Result<Option<Position>> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .next_mate_alignment_start_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::NextMateAlignmentStart),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
        .map(Position::new)
    }

    fn read_template_size(&mut self) -> io::Result<i32> {
        self.compression_header
            .data_series_encoding_map()
            .template_size_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::TemplateSize),
                )
            })
            .and_then(|encoding| {
                decode_itf8(
                    encoding,
                    &mut self.core_data_reader,
                    &mut self.external_data_readers,
                )
            })
    }

    fn read_distance_to_next_fragment(&mut self) -> io::Result<usize> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .distance_to_next_fragment_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::DistanceToNextFragment),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_tag_data(&mut self) -> io::Result<sam::record::Data> {
        use bam::reader::record::data::field::get_value;
        use sam::record::data::Field;

        let tag_line = self.read_tag_line()?;

        let tag_keys = self
            .compression_header
            .preservation_map()
            .tag_ids_dictionary()
            .get(tag_line)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid tag line"))?;

        let tag_encoding_map = self.compression_header.tag_encoding_map();

        let mut fields = Vec::with_capacity(tag_keys.len());

        for key in tag_keys {
            let id = key.id();
            let encoding = tag_encoding_map.get(&id).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingTagEncoding(*key),
                )
            })?;

            let data = decode_byte_array(
                encoding,
                &mut self.core_data_reader,
                &mut self.external_data_readers,
                None,
            )?;

            let mut data_reader = &data[..];
            let value = get_value(&mut data_reader, key.ty())?;

            let field = Field::new(key.tag(), value);
            fields.push(field);
        }

        sam::record::Data::try_from(fields)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read_tag_line(&mut self) -> io::Result<usize> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .tag_ids_encoding();

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_mapped_read(
        &mut self,
        record: &mut Record,
        flags: Flags,
        read_length: usize,
    ) -> io::Result<()> {
        let feature_count = self.read_number_of_read_features()?;

        let mut prev_position = 0;

        for _ in 0..feature_count {
            let feature = self.read_feature(prev_position)?;
            prev_position = usize::from(feature.position());
            record.add_feature(feature);
        }

        record.mapping_quality = self.read_mapping_quality()?;

        if flags.are_quality_scores_stored_as_array() {
            record.quality_scores.as_mut().reserve(read_length);

            for _ in 0..read_length {
                let score = self.read_quality_score()?;
                record.quality_scores.push(score);
            }
        }

        Ok(())
    }

    fn read_number_of_read_features(&mut self) -> io::Result<usize> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .number_of_read_features_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::NumberOfReadFeatures),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_feature(&mut self, prev_position: usize) -> io::Result<Feature> {
        use feature::Code;

        let code = self.read_feature_code()?;

        let delta = self.read_feature_position()?;
        let position = Position::try_from(prev_position + delta)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        match code {
            Code::Bases => {
                let bases = self.read_stretches_of_bases()?;
                Ok(Feature::Bases(position, bases))
            }
            Code::Scores => {
                let quality_scores = self.read_stretches_of_quality_scores()?;
                Ok(Feature::Scores(position, quality_scores))
            }
            Code::ReadBase => {
                let base = self.read_base()?;
                let quality_score = self.read_quality_score()?;
                Ok(Feature::ReadBase(position, base, quality_score))
            }
            Code::Substitution => {
                let code = self.read_base_substitution_code()?;
                Ok(Feature::Substitution(position, code))
            }
            Code::Insertion => {
                let bases = self.read_insertion()?;
                Ok(Feature::Insertion(position, bases))
            }
            Code::Deletion => {
                let len = self.read_deletion_length()?;
                Ok(Feature::Deletion(position, len))
            }
            Code::InsertBase => {
                let base = self.read_base()?;
                Ok(Feature::InsertBase(position, base))
            }
            Code::QualityScore => {
                let score = self.read_quality_score()?;
                Ok(Feature::QualityScore(position, score))
            }
            Code::ReferenceSkip => {
                let len = self.read_reference_skip_length()?;
                Ok(Feature::ReferenceSkip(position, len))
            }
            Code::SoftClip => {
                let bases = self.read_soft_clip()?;
                Ok(Feature::SoftClip(position, bases))
            }
            Code::Padding => {
                let len = self.read_padding()?;
                Ok(Feature::Padding(position, len))
            }
            Code::HardClip => {
                let len = self.read_hard_clip()?;
                Ok(Feature::HardClip(position, len))
            }
        }
    }

    fn read_feature_code(&mut self) -> io::Result<feature::Code> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .read_features_codes_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::ReadFeaturesCodes),
                )
            })?;

        decode_byte(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|id| {
            feature::Code::try_from(id).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        })
    }

    fn read_feature_position(&mut self) -> io::Result<usize> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .in_read_positions_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::InReadPositions),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_stretches_of_bases(&mut self) -> io::Result<Vec<Base>> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .stretches_of_bases_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::StretchesOfBases),
                )
            })?;

        let raw_bases = decode_byte_array(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
            None,
        )?;

        raw_bases
            .into_iter()
            .map(|n| Base::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
            .collect()
    }

    fn read_stretches_of_quality_scores(&mut self) -> io::Result<Vec<Score>> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .stretches_of_quality_scores_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(
                        DataSeries::StretchesOfQualityScores,
                    ),
                )
            })?;

        let scores = decode_byte_array(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
            None,
        )?;

        scores
            .into_iter()
            .map(|n| Score::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
            .collect()
    }

    fn read_base(&mut self) -> io::Result<Base> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .bases_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::Bases),
                )
            })?;

        decode_byte(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| Base::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_quality_score(&mut self) -> io::Result<Score> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .quality_scores_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::QualityScores),
                )
            })?;

        decode_byte(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| Score::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_base_substitution_code(&mut self) -> io::Result<substitution::Value> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .base_substitution_codes_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::BaseSubstitutionCodes),
                )
            })?;

        decode_byte(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .map(substitution::Value::Code)
    }

    fn read_insertion(&mut self) -> io::Result<Vec<Base>> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .insertion_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::Insertion),
                )
            })?;

        let raw_bases = decode_byte_array(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
            None,
        )?;

        raw_bases
            .into_iter()
            .map(|n| Base::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
            .collect()
    }

    fn read_deletion_length(&mut self) -> io::Result<usize> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .deletion_lengths_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::DeletionLengths),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_reference_skip_length(&mut self) -> io::Result<usize> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .reference_skip_length_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::ReferenceSkipLength),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_soft_clip(&mut self) -> io::Result<Vec<Base>> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .soft_clip_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::SoftClip),
                )
            })?;

        let raw_bases = decode_byte_array(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
            None,
        )?;

        raw_bases
            .into_iter()
            .map(|n| Base::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
            .collect()
    }

    fn read_padding(&mut self) -> io::Result<usize> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .padding_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::Padding),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_hard_clip(&mut self) -> io::Result<usize> {
        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .hard_clip_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::HardClip),
                )
            })?;

        decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
    }

    fn read_mapping_quality(&mut self) -> io::Result<Option<sam::record::MappingQuality>> {
        use sam::record::mapping_quality::MISSING;

        let encoding = self
            .compression_header
            .data_series_encoding_map()
            .mapping_qualities_encoding()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    ReadRecordError::MissingDataSeriesEncoding(DataSeries::MappingQualities),
                )
            })?;

        let n = decode_itf8(
            encoding,
            &mut self.core_data_reader,
            &mut self.external_data_readers,
        )
        .and_then(|n| u8::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))?;

        match n {
            MISSING => Ok(None),
            _ => Ok(sam::record::MappingQuality::new(n)),
        }
    }

    fn read_unmapped_read(
        &mut self,
        record: &mut Record,
        flags: Flags,
        read_length: usize,
    ) -> io::Result<()> {
        record.bases.as_mut().reserve(read_length);

        for _ in 0..read_length {
            let base = self.read_base()?;
            record.bases.push(base);
        }

        if flags.are_quality_scores_stored_as_array() {
            record.quality_scores.as_mut().reserve(read_length);

            for _ in 0..read_length {
                let score = self.read_quality_score()?;
                record.quality_scores.push(score);
            }
        }

        Ok(())
    }
}

fn decode_byte<CDR, EDR>(
    encoding: &Encoding,
    core_data_reader: &mut BitReader<CDR>,
    external_data_readers: &mut ExternalDataReaders<EDR>,
) -> io::Result<u8>
where
    CDR: Buf,
    EDR: Buf,
{
    match encoding {
        Encoding::External(block_content_id) => {
            let src = external_data_readers
                .get_mut(block_content_id)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        ReadRecordError::MissingExternalBlock(*block_content_id),
                    )
                })?;

            if !src.has_remaining() {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
            }

            Ok(src.get_u8())
        }
        Encoding::Huffman(alphabet, bit_lens) => {
            if alphabet.len() == 1 {
                Ok(alphabet[0] as u8)
            } else {
                let decoder = CanonicalHuffmanDecoder::new(alphabet, bit_lens);
                decoder.decode(core_data_reader).map(|i| i as u8)
            }
        }
        _ => todo!("decode_byte: {:?}", encoding),
    }
}

fn decode_itf8<CDR, EDR>(
    encoding: &Encoding,
    core_data_reader: &mut BitReader<CDR>,
    external_data_readers: &mut ExternalDataReaders<EDR>,
) -> io::Result<i32>
where
    CDR: Buf,
    EDR: Buf,
{
    match encoding {
        Encoding::External(block_content_id) => {
            let src = external_data_readers
                .get_mut(block_content_id)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        ReadRecordError::MissingExternalBlock(*block_content_id),
                    )
                })?;

            get_itf8(src)
        }
        Encoding::Huffman(alphabet, bit_lens) => {
            if alphabet.len() == 1 {
                Ok(alphabet[0])
            } else {
                let decoder = CanonicalHuffmanDecoder::new(alphabet, bit_lens);
                decoder.decode(core_data_reader)
            }
        }
        Encoding::Beta(offset, len) => core_data_reader.read_u32(*len).map(|i| (i as i32 - offset)),
        _ => todo!("decode_itf8: {:?}", encoding),
    }
}

fn decode_byte_array<CDR, EDR>(
    encoding: &Encoding,
    core_data_reader: &mut BitReader<CDR>,
    external_data_readers: &mut ExternalDataReaders<EDR>,
    buf: Option<Vec<u8>>,
) -> io::Result<Vec<u8>>
where
    CDR: Buf,
    EDR: Buf,
{
    match encoding {
        Encoding::External(block_content_id) => {
            let src = external_data_readers
                .get_mut(block_content_id)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        ReadRecordError::MissingExternalBlock(*block_content_id),
                    )
                })?;

            let mut buf = buf.unwrap();

            if src.remaining() < buf.len() {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
            }

            src.copy_to_slice(&mut buf);

            Ok(buf)
        }
        Encoding::ByteArrayLen(len_encoding, value_encoding) => {
            let len = decode_itf8(len_encoding, core_data_reader, external_data_readers)?;

            let buf = vec![0; len as usize];
            let value = decode_byte_array(
                value_encoding,
                core_data_reader,
                external_data_readers,
                Some(buf),
            )?;

            Ok(value)
        }
        Encoding::ByteArrayStop(stop_byte, block_content_id) => {
            let src = external_data_readers
                .get_mut(block_content_id)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        ReadRecordError::MissingExternalBlock(*block_content_id),
                    )
                })?;

            let len = match src.chunk().iter().position(|&b| b == *stop_byte) {
                Some(i) => i,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "missing byte array stop byte",
                    ))
                }
            };

            let mut buf = vec![0; len];
            src.copy_to_slice(&mut buf);

            // Discard the stop byte.
            src.advance(1);

            Ok(buf)
        }
        _ => todo!("decode_byte_array: {:?}", encoding),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_byte() -> io::Result<()> {
        fn t(encoding: &Encoding, expected: u8) -> io::Result<()> {
            let core_data = [0b10000000];
            let mut core_data_reader = BitReader::new(&core_data[..]);

            let external_data = [0x0d];
            let mut external_data_readers = ExternalDataReaders::new();
            external_data_readers.insert(1, &external_data[..]);

            let actual = decode_byte(encoding, &mut core_data_reader, &mut external_data_readers)?;

            assert_eq!(expected, actual);

            Ok(())
        }

        t(&Encoding::External(1), 0x0d)?;
        t(&Encoding::Huffman(vec![0x4e], vec![0]), 0x4e)?;

        Ok(())
    }

    #[test]
    fn test_decode_itf8() -> io::Result<()> {
        fn t(encoding: &Encoding, expected: i32) -> io::Result<()> {
            let core_data = [0b10000000];
            let mut core_data_reader = BitReader::new(&core_data[..]);

            let external_data = [0x0d];
            let mut external_data_readers = ExternalDataReaders::new();
            external_data_readers.insert(1, &external_data[..]);

            let actual = decode_itf8(encoding, &mut core_data_reader, &mut external_data_readers)?;

            assert_eq!(expected, actual);

            Ok(())
        }

        t(&Encoding::External(1), 13)?;
        t(&Encoding::Huffman(vec![0x4e], vec![0]), 0x4e)?;
        t(&Encoding::Beta(1, 3), 3)?;

        Ok(())
    }

    #[test]
    fn test_decode_byte_array() -> io::Result<()> {
        fn t(external_data: &[u8], encoding: &Encoding, expected: &[u8]) -> io::Result<()> {
            let core_data = [];
            let mut core_data_reader = BitReader::new(&core_data[..]);

            let mut external_data_readers = ExternalDataReaders::new();
            external_data_readers.insert(1, external_data);

            let actual = decode_byte_array(
                encoding,
                &mut core_data_reader,
                &mut external_data_readers,
                None,
            )?;

            assert_eq!(expected, actual);

            Ok(())
        }

        let len_encoding = Encoding::External(1);
        let value_encoding = Encoding::External(1);
        t(
            &[0x04, 0x6e, 0x64, 0x6c, 0x73],
            &Encoding::ByteArrayLen(Box::new(len_encoding), Box::new(value_encoding)),
            b"ndls",
        )?;

        t(
            &[0x6e, 0x64, 0x6c, 0x73, 0x00],
            &Encoding::ByteArrayStop(0x00, 1),
            b"ndls",
        )?;

        assert!(matches!(
            t(&[0x6e, 0x64, 0x6c, 0x73], &Encoding::ByteArrayStop(0x00, 1), b""),
            Err(e) if e.kind() == io::ErrorKind::InvalidData
        ));

        Ok(())
    }
}
