# Changelog

## 0.32.0 - 2023-06-08

### Added

  * cram/reader: Add builder (`cram::reader::Builder`).

### Changed

  * cram: Update to bzip2 0.4.4 ([#174]).

  * cram/reader: Add a reference sequence repository as context.

    `Reader::records` and `Reader::query` no longer accepts a reference
    sequence repository but uses the one attached to the reader. Use
    `cram::reader::Builder` to set the appropriate `fasta::Repository`.

[#174]: https://github.com/zaeleus/noodles/pull/174

## 0.31.0 - 2023-06-01

### Changed

  * cram: Update to async-compression 0.4.0.

  * cram: Update to libdeflater 0.14.0 (libdeflate 1.18).

## 0.30.0 - 2023-05-18

### Added

  * cram/reader/record: Add Elias gamma decoder for integers.

## 0.29.0 - 2023-05-11

### Changed

  * cram: Sync dependencies.

## 0.28.0 - 2023-05-04

### Changed

  * cram: Sync dependencies.

## 0.27.0 - 2023-04-27

### Changed

  * cram: Update to libdeflater 0.13.0.

## 0.26.0 - 2023-04-06

### Added

  * cram: Add an indexed reader (`cram::IndexedReader`).

### Changed

  * cram: Update to bitflags 2.0.2.

  * cram/reader: Change `Reader::read_file_header` to return a parsed header
    (`sam::Header`).

    This no longer returns a raw string.

## 0.25.0 - 2023-03-14

### Changed

  * cram: Sync dependencies.

## 0.24.0 - 2023-03-03

### Changed

  * cram: Sync dependencies.

## 0.26.0 - 2023-04-06

  * cram: Add `libdeflate` feature to enable using libdeflate for gzip streams.

## 0.23.0 - 2023-02-03

### Added

  * cram/crai/record: Implement `std::error::Error::source` for errors.

### Changed

  * cram: Raise minimum supported Rust version (MSRV) to 1.64.0.

  * cram/record: Return `Name`-`Map<ReferenceSequence>` pair from
    `Record::reference_sequence` and `Record::mate_reference_sequence`.

  * cram/record/builder: Change `Builder::add_tag` to receive a `Tag`-`Value`
    pair.

### Removed

  * cram/data_container/compression_header/preservation_map/tag_ids_dictionary/key:
    Remove `From<&sam::record::data::Field>`.

    Use `Key::new` instead.

  * cram/record/feature/code: Remove `TryFrom<char>` and `TryFromCharError`.

    These were deprecated in noodles-cram 0.13.0. Use `TryFrom<u8>` and
    `TryFromByteError`, respectively, instead.

  * cram/record/feature/code: Remove `From<Code>` for `char`.

    This was deprecated in noodles-cram 0.13.0. Convert to a `u8`
    instead.

  * cram/record/resolve: Remove `resolve_features`.

    This was deprecated in noodles-cram 0.16.0. Use
    `Features::try_into_cigar` instead.

## 0.22.1 - 2022-11-29

### Changed

  * cram: Sync dependencies.

## 0.22.0 - 2022-11-18

### Added

  * cram/async/reader: Add query stream (`Reader::query`).

### Changed

  * cram/reader/container/block: Treat blocks with no data as empty, regardless
    of compression method ([#131]).

  * cram/reader/header_container: Require blocks to be either uncompressed or
    gzipped.

    This was clarified in [samtools/hts-specs@4b61c4d].

[#131]: https://github.com/zaeleus/noodles/issues/131
[samtools/hts-specs@4b61c4d]: https://github.com/samtools/hts-specs/commit/4b61c4d7e40748b274478fd481bbfe60592a969b

### Fixed

  * cram/reader/record: Read quality scores filled with `0xff` as missing
    ([#136]).

[#136]: https://github.com/zaeleus/noodles/issues/136

## 0.21.0 - 2022-10-28

### Added

  * cram/async: Add async writer (`cram::AsyncWriter`).

### Fixed

  * cram/reader/header_container: Truncate data by header length.

    This is required if the header text is padded rather than using a new block
    for padding.

## 0.20.0 - 2022-10-20

### Added

  * cram: Add CRAM 3.1 write support ([#107]).

    The format of CRAM 3.1 is structurally the same as 3.0, but 3.1 includes
    additional block content compression methods: rANS Nx16, adaptive
    arithmetic coder, fqzcomp, and name tokenizer.

  * cram/codecs: Add fqzcomp encoder.

  * cram/codecs: Increase visibilities of `aac::Flags`, `rans_4x8::Order`, and
    `rans_nx16::Flags`.

  * cram/codecs/name_tokenizer/encode: Add `DIGITS0`, `DIGITS`, `DELTA`,
    `DELTA0`, and `MATCH` tokens.

### Changed

  * cram: Raise minimum supported Rust version (MSRV) to 1.62.0.

  * cram/writer: Calculate and write missing MD5 checksums (`M5`) in the
    reference sequence dictionary.

### Fixed

  * cram/codecs/aac/encode: Fix serialization of the max symbol for order-0 and
    1 encoders.

  * cram/codecs/aac/encode: Disable bit packing if the input has more than 16
    symbols.

  * cram/codecs/fqzcomp/decode: Fix reading records with varying lengths.

  * cram/codecs/name_tokenizer/decode: Add missing mutable `DUP` reader.

  * cram/codecs/name_tokenizer/decode: Fix setting `TYPE` reader when reading
    token byte streams.

  * cram/codecs/name_tokenizer/encode: Strip final separator before splitting
    names.

  * cram/codecs/rans_4x8/encode: Fix run-length calculation when writing
    frequencies.

  * cram/codecs/rans_4x8/encode: Avoid scaling frequencies to 0 during
    normalization.

  * cram/codecs/rans_4x8/encode: Ensure the sum of the normalized frequencies
    equals the scaling factor.

  * cram/codecs/rans_4x8/encode/order_1: Fix order of rANS state
    renormalizations.

  * cram/codecs/rans_nx16/encode: Fix run-length calculation when writing
    frequencies.

  * cram/codecs/rans_nx16/encode: Avoid scaling frequencies to 0 during
    normalization.

  * cram/codecs/rans_nx16/encode: Handle 0 RLE symbols.

  * cram/codecs/rans_nx16/encode: Disable rANS encoding when the input is too
    small.

  * cram/codecs/rans_nx16/encode: Disable bit packing if the input has more
    than 16 symbols.

  * cram/codecs/rans_nx16/encode: Ensure the sum of the normalized frequencies
    equals the scaling factor.

  * cram/codecs/rans_nx16/encode/order_1: Fix order of rANS state
    renormalizations.

[#107]: https://github.com/zaeleus/noodles/issues/107

## 0.19.0 - 2022-09-29

### Added

  * cram/codecs: Add rANS Nx16, adaptive arithmetic coder, and name tokenizer
    encoders.

  * cram/data_container: Add block content-encoder map
    (`BlockContentEncoderMap`) to assign encoders to block contents (core, data
    series, and tag values).

  * cram/writer/builder: Add setter for block content-encoder map.

### Changed

  * cram: Increased visibilities of `cram::writer`,
    `data_series_encoding_map::DataSeries`, and `tag_ids_dictionary::Key`.

  * cram/writer/builder: Change `build` to `build_with_writer`.

    `writer::Builder` no longer holds the inner reader.

### Fixed

  * cram/container/block: Add terminators (`NUL`) to decoded names.

### Removed

  * cram/writer: Remove `Writer::builder`.

    Use `cram::writer::Builder` directly instead.

## 0.18.0 - 2022-08-16

### Changed

  * cram: Raise minimum supported Rust version (MSRV) to 1.59.0.

## 0.17.0 - 2022-07-05

### Changed

  * cram: Sync dependencies.

## 0.16.0 - 2022-06-08

### Added

  * cram/async/reader: Add records stream (`AsyncReader::records`).

### Changed

  * cram/async/reader/container/header: Verify the expected CRC32 checksum of
    the container header.

  * cram/data_container/slice: Template size for records with downstream mates
    is now calculated from the leftmost to rightmost base, regardless of
    orientation.

  * cram/data_container/slice: Verify reference sequence checksum.

    § 11 "Reference sequences" (2021-11-15): "All CRAM reader implementations
    are expected to check for reference MD5 checksums and report any missing or
    mismatching entries."

  * cram/data_container/slice/builder: Match mate by read name.

  * cram/reader/container/block: Verify the expected CRC32 checksum of the
    block.

  * cram/reader/container/header: Verify the expected CRC32 checksum of the
    container header.

  * cram/record/convert: Change `Record::try_into_sam_record` to
    `Record::try_into_alignment_record`.

    This now takes a `&sam::alignment::Record`.

### Fixed

  * cram/data_container/builder: Disable encoding alignment start positions as
    deltas when the container has multi-reference slices.

  * cram/data_container/slice: Resolve read group ID.

    This adds the `RG` field with the corresponding read group ID to data.

  * cram/data_container/slice: Resolve read name if none is set.

### Deprecated

  * cram/record/resolve: Deprecate `resolve_features`.

    Use `Features::try_into_cigar` instead.

### Removed

  * cram/record: Remove `sam::AlignmentRecord`.

    Convert to an alignment record (`Record::try_into_alignment_record`)
    instead.

## 0.15.0 - 2022-04-14

### Added

  * cram/reader: Add query iterator (`Reader::query`).

  * cram/reader/data_container/compression_header/encoding: Add Golomb and
    Golomb-Rice encoding decoders.

  * cram/writer/data_container/compression_header/encoding: Add Golomb and
    Golomb-Rice encoding writers.

    These see no usage in practice, but they are still in the spec (§ 13.8 and
    § 13.9, respectively). They are only added for completeness.

### Changed

  * cram/data_container/slice/builder: Slices can now be multi-reference.

### Fixed

  * cram/container: Return error (`TryFromSamHeaderError::InvalidHeaderLength`)
    when header length is too large.

## 0.14.0 - 2022-03-29

### Added

  * cram/reader: Implement `sam::AlignmentReader`.

  * cram/record: Add conversion from an alignment record to a CRAM record.

  * cram/record/features: Add conversion from SAM CIGAR operations.

  * cram/writer: Implement `sam::AlignmentWriter`.

### Changed

  * cram/crai/record: Change reference sequence ID to an `Option<usize>`;
    alignment start, `Option<Position>`; and alignment span, `usize`.

  * cram/record: Move `mapping_quality`, `read_name`, and `quality_scores` to
    the implementation of `sam::AlignmentRecord`.

  * cram/record: Rename `flags` to `cram_flags`.

  * cram/record: Wrap read name as `sam::record::ReadName`.

  * cram/record: Wrap quality scores as `sam::record::QualityScores`.

  * cram/record: Wrap sequence as `sam::record::Sequence`.

  * cram/record: Change tags to `sam::record::Data`.

  * cram/record: Change alignment start and next mate alignment start to a
    `Position`.

  * cram/record: Change distance to next fragment to an `Option<usize>`.

    This distance can only be forward (_CRAM format specification (version
    3.0)_ (2021-10-15) § 8.4 "Compression header block": "... NF can only refer
    to a record later within [the] slice.") and is only set when the "has mate
    downstream" flag is set.

  * cram/record: Change read group ID to an `Option<usize>`.

  * cram/record: Change reference sequence ID and next fragment reference
    sequence ID to an `Option<usize>`.

  * cram/record/convert: Accept `sam::Header` instead of `ReferenceSequences`.

  * cram/record/feature: Change underlying data types:

      * bases as `sam::record::sequence::Base`,
      * lengths as `usize`,
      * positions as `Position`,
      * quality scores as `sam::record::quality_scores::Score`, and
      * substitution as `substitution::Value`.

  * cram/record/resolve: Merge adjacent CIGAR operations.

    For example, instead of 1M1M2M, this is now 4M.

  * cram/record/tag/key: Change tag to a `sam::record::data::field::Tag`.

  * cram/writer: Make a reference sequence repository optional.

    Use `Builder::set_reference_sequence_repository` to set one.

  * cram/writer: A writer no longer has to be constructed with a header.

    However, `Writer::write_record` and `Writer::try_finish` must be called
    with a reference to a `sam::Header` now.

### Fixed

  * cram/data_container/slice: Only attempt to resolve quality scores for
    mapped records.

  * cram/data_container/slice/builder: Detach all records before writing.

    The slice builder does not yet calculate mate distances, so all records are
    currently written as detached.

  * cram/data_container/slice/builder: Normalize sequence for MD5 digest
    calculation.

  * cram/reader/data_container: Disallow unexpected block content types when
    reading a data container.

  * cram/record/convert: Create read group (`RG`) data field if a read group ID
    is set.

  * cram/record/convert: Only convert CIGAR to features for mapped records.

    Unmapped records cannot have features.

  * cram/record/resolve: Handle `Scores` feature when resolving quality scores.

  * cram/writer: Avoid possible wrapping when casting `usize` to `i32`.

### Removed

  * cram/record: Remove `ReadGroupId`.

    Use `usize` instead.

  * cram/record: Remove `Tag`.

    Use `sam::record::data::Field` instead.

  * cram/writer: Remove `Drop`.

    A call to `Writer::try_finish` must be made before the writer is dropped.

## 0.13.0 - 2022-03-02

### Added

  * cram/data_container/slice: Add slice records resolver.

    This resolves mates, read names, bases, and quality scores.

  * cram/record: Add features wrapper (`Features`).

  * cram/record: Implement `sam::RecordExt`.

  * cram/record/feature/code: Implement `TryFrom<u8>`.

  * cram/record/feature/code: Add conversion to `u8`.

  * cram/record/resolve: Add quality scores resolver.

  * cram/record/resolve: Allow base resolution when there is no reference
    sequence available.

    Even when both an external reference sequence and embedded reference
    sequence are missing, it is still possible to resolve bases.

  * cram/writer: Add builder.

  * cram/writer/builder: Add options to change applicable preservation map
    values, i.e., whether to preserve read names and whether to encode record
    alignment start positions as deltas.

### Changed

  * cram/data_container/builder: Decrease max slice count to 1.

  * cram/data_container/slice/builder: Increase max record count to 10240.

  * cram/reader: `records` uses a `fasta::Repository` instead of
    `&[fasta::Record]`.

    This avoids having to do an initial read of all reference sequences. It
    also now requires the SAM header.

  * cram/reader/records: Resolve bases and quality scores when reading records.

    This previously only resolved mates and read names.

  * cram/record/convert: `Record::try_into_sam_record` now assumes the record
    is fully resolved before conversion.

  * cram/record/read_group_id: Change underlying type to a `usize`.

    Use `From<usize>` instead of `From<i32>`.

  * cram/record/resolve: `resolve_bases` takes a `sam::record::Position`
    instead of an `i32` for `alignment_start`.

  * cram/writer: Use reference sequence repository (`fasta::Repository`) and
    SAM header (`sam::Header`) when writing records.

### Deprecated

  * cram/record/feature/code: Deprecate fallible conversion from `char`.

    This also deprecates `TryFromCharError`. Use `TryFrom<u8>` instead.

  * cram/record/feature/code: Deprecate conversion to `char`.

    Convert to a `u8` instead.

### Fixed

  * cram/container: Only set alignment start and span when the container has
    single-reference slices of the same reference.

  * cram/container: Ensure all slices in the container have the same reference
    sequence ID.

  * cram/data_container/slice/builder: Update substitution codes before writing
    records.

  * cram/data_container/slice/builder: Only set alignment start and span when
    the slice is a single-reference slice.

  * cram/data_container/compression_header/preservation_map/substitution_matrix/builder:
    Read read base from read bases.

  * cram/record/convert: Only resolve features when the record is mapped.

  * cram/writer/record: Fix data series type when the features code encoding is
    missing.

### Removed

  * cram/data_container/slice: Remove `resolve_mates`.

    Use `Slice::resolve_records` instead.

  * cram/record/read_group_id: Remove conversion to and from `i32`.

    Convert from `usize` instead.

  * cram/record/resolve: Remove `resolve_bases`.

    Base resolution can only be guaranteed when the slice is available, as it
    may contain an embedded reference sequence.

    Use `Slice::resolve_records` instead.

## 0.12.0 - 2022-02-17

### Added

  * cram: Set minimum supported Rust version (MSRV) to 1.56.0 in package
    manifest.

  * cram/container/block/compression_method: Add decoder support for CRAM 3.1
    (draft) block compression methods: rANS Nx16, adaptive arithmetic coder,
    fqzcomp, and name tokenizer.

  * cram/record/resolve: Add read base (`B`) base resolver.

### Changed

  * cram: Write 0 for alignment start and alignment span for slices with
    unmapped records when indexing.

  * cram/data_container/slice: The mate resolver (`Slice::resolve_mates`) is
    now fallible.

  * cram/record: `Record::alignment_end` now returns
    `Option<io::Result<sam::record::Position>>`.

### Fixed

  * cram/data_container/slice: Fix template size calculation when resolving
    mates.

  * cram/writer/data_container/compression_header/data_series_encoding_map:
    Avoid truncating data length.

## 0.11.0 - 2022-01-27

### Added

  * cram/record/resolve: Add bases (`b`) base resolver.

### Fixed

  * cram/async/reader/record: Read feature code as byte.

  * cram/huffman: Fix read length from bit reader.

  * cram/reader/record: Read feature code as byte.

    This was incorrectly being read as an ITF-8 value.

## 0.10.0 - 2022-01-13

### Added

  * cram/record: Mapping quality is now stored as an `Option`.

    Valid mapping qualities are between 0 and 254, inclusive (`Some`). A
    mapping quality of 255 is considered to be missing (`None`).

## 0.9.0 - 2021-12-16

### Changed

  * cram: Update to [md-5 0.10.0].

[md-5 0.10.0]: https://crates.io/crates/md-5/0.10.0

## 0.8.3 - 2021-12-09

### Fixed

  * cram: Sync dependencies.

## 0.8.2 - 2021-12-02

### Fixed

  * cram: Require tokio's `fs` feature as a dependency ([#62]).

[#62]: https://github.com/zaeleus/noodles/issues/62

## 0.8.1 - 2021-11-18

### Fixed

  * cram: Sync dependencies.

## 0.8.0 - 2021-11-11

### Added

  * cram/crai: Add convenience write function (`crai::write`).

  * cram/crai/async: Add async writer (`crai::AsyncWriter`).

  * cram/crai/async: Add convenience write function (`crai::r#async::write`).

### Changed

  * cram: Update to Rust 2021.

## 0.7.0 - 2021-10-16

### Added

  * cram/data_container/compression_header/data_series_encoding_map/
    data_series: Add legacy TC and TN data series.

    These are no longer used in CRAM 3.0 but still need to be handled. See
    samtools/hts-specs@9a0513783826516fb8086ecf82d13631a2292f75.

  * cram/record/resolve: Handle reference skip feature in sequence resolver.

## 0.6.1 - 2021-10-02

### Fixed

  * cram: Sync dependencies.

## 0.6.0 - 2021-10-01

### Added

  * cram/reader: Add common methods to access the underlying reader: `get_ref`,
    `get_mut`, and `into_inner`.

### Fixed

  * cram/rans/decode/order_1: Fix overflow when reading frequencies.

## 0.5.1 - 2021-09-23

### Fixed

  * cram/async/reader/container/header: Fix reading starting position on the
    reference.

  * cram/async/reader/data_container/slice/header: Fix reading alignment start.

## 0.5.0 - 2021-09-19

### Added

  * cram/crai/record: Implement `Display`.

  * cram/reader: Add data container reader.

    This can be used to manually read records from slices.

  * cram/record: Add conversion to SAM record
    (`cram::Record::try_into_sam_record`).

### Changed

  * cram/record: Change alignment start to a `sam::record::Position`.

  * cram/record: Change next mate alignment start to a `sam::record::Position`.

  * cram/record/resolve: Pass compression header rather than substitution
    matrix.

    The compression header includes the substitution matrix in the preservation
    map.

### Fixed

  * cram/async/reader/data_container/slice/header: Read remainder of stream as
    optional tags.

  * cram/reader/container: Avoid casts that may truncate.

  * cram/reader/data_container/compression_header/encoding: Avoid casts that
    may truncate.

    Buffer sizes that convert from `Itf8` to `usize` now check whether they are
    in range.

  * cram/reader/data_container/slice/header: Read remainder of stream as
    optional tags.

  * cram/record/resolve: Increment feature position with operations that
    consume the read.

  * cram/record/resolve: Include last feature position.

## 0.4.0 - 2021-09-01

### Added

  * cram/async/reader: Add data container reader.

  * cram/reader: Add data container reader.

    This can be used to manually read records from slices.

### Changed

  * cram/record: `Record::read_length` is now stored as a `usize`.

### Fixed

  * cram/reader/data_container/compression_header: Avoid casts that may
    truncate.

    Buffer sizes that convert from `Itf8` to `usize` now check whether they are
    in range.

## 0.3.0 - 2021-08-19

### Added

  * cram/async: Add async header reader (`cram::AsyncReader`).

    This is a partial async CRAM reader that can only read the file definition
    and file header.

  * cram/crai/async: Add async reader (`crai::AsyncReader`).

  * cram/crai/async: Add async writer (`crai::AsyncWriter`).

    Async I/O can be enabled with the `async` feature.

## 0.2.2 - 2021-08-11

### Fixed

  * cram: Sync dependencies.

## 0.2.1 - 2021-07-30

### Fixed

  * cram: Sync dependencies.

## 0.2.0 - 2021-07-21

### Added

  * cram/record/tag: Add conversion from `Tag` to `sam::record::data::Field`.

### Fixed

  * cram: Fixed documentation link in package manifest ([#31]).

[#31]: https://github.com/zaeleus/noodles/issues/31

## 0.1.0 - 2021-07-14

  * cram: Initial release.
