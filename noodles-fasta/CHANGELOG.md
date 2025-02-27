# Changelog

## 0.24.0 - 2023-06-08

### Added

  * fasta/reader: Add a sequence reader (`fasta::reader::sequence::Reader`)
    ([#101]).

    This is created by calling `fasta::Reader::sequence_reader`. It is used for
    lower-level reading of the sequence.

[#101]: https://github.com/zaeleus/noodles/issues/101

## 0.23.0 - 2023-06-01

### Changed

  * fasta: Sync dependencies.

### Deprecated

  * fasta/fai/record: Deprecate `Record::len`.

    Use `Record::length` instead.

## 0.22.0 - 2023-05-04

### Changed

  * fasta/fai/record: Accept `Into<String>` for the record name.

## 0.21.0 - 2023-04-27

### Changed

  * fasta: Sync dependencies.

## 0.20.0 - 2023-03-14

### Added

  * fasta/indexed_reader: Add getter for index (`IndexedReader::index`).

## 0.19.0 - 2023-03-03

### Changed

  * fasta/reader: Improve performance of querying small regions ([#146]).

    The reader no longer needs to read the entire sequence when querying.

  * fasta/reader/builder: Add `bgz` as a known BGZF extension ([#144]).

[#144]: https://github.com/zaeleus/noodles/pull/144
[#146]: https://github.com/zaeleus/noodles/issues/146

## 0.18.0 - 2023-02-03

### Added

  * fasta/fai/record: Implement `std::error::Error::source` for `ParseError`.

### Changed

  * fasta: Raise minimum supported Rust version (MSRV) to 1.64.0.

### Removed

  * fasta/fai/record: Remove `Record::reference_sequence_name`.

    This was deprecated in noodles-fasta 0.3.0. Use `Record::name`
    instead.

  * fasta/record: Remove `Record::reference_sequence_name`.

    This was deprecated in noodles-fasta 0.2.0. Use `Record::name`
    instead.

  * fasta/record/definition: Remove
    `Definition::reference_sequence_name`.

    This was deprecated in noodles-fasta 0.3.0. Use `Definition::name`
    instead.

  * fasta/record/definition: Remove
    `ParseError::MissingReferenceSequenceName`.

    This was deprecated in noodles-fasta 0.6.0. Use
    `ParseError::MissingName` instead.


## 0.17.0 - 2022-11-18

### Changed

  * fasta: Sync dependencies.

## 0.16.0 - 2022-10-28

### Added

  * fasta/writer/builder: Implement `Default`.

### Changed

  * fasta/writer/builder: `Builder` no longer holds a writer.

### Removed

  * fasta/writer: Remove `Writer::builder`.

    Use `writer::Builder::default` instead.

  * fasta/writer/builder: Remove `Builder::build`.

    Use `Builder::build_with_writer` instead.

## 0.15.0 - 2022-10-20

### Changed

  * fasta: Sync dependencies.

### Unreleased

## Changed

  * fasta: Split indexed reader from reader.

    `reader::Builder` no longer attempts to load associated indices. This
    functionality is separated into `IndexedReader`, which now guarantees
    associated indices are loaded for querying.

    Changes usages of `fasta::reader::Builder` to
    `fasta::indexed_reader::Builder` if it is known querying is necessary.

## 0.14.0 - 2022-09-29

### Added

  * fasta/reader: Add common methods to access the underlying reader
    (`Reader::get_ref`, `Reader::get_mut`, and `Reader::into_inner`).

  * fasta/reader: Add builder (`fasta::reader::Builder`).

    The builder is able to construct a reader from a path
    (`Builder::build_from_path`), which can open raw FASTA files (`*.fa`) and
    bgzipped FASTA (`*.fa.gz`) files. If an associated index (`*.fai`) exists,
    it is loaded to allow querying.

### Changed

  * fasta/reader: `Reader::query` no longer takes a `fai::Index` as input.

    Use an indexed reader via `reader::Builder` to load or set an associated
    index instead.

### Removed

  * fasta/reader: Remove `seek` and `virtual_position` delegates.

    Use the inner reader instead.

  * fasta/repository/adapters/indexed_reader: Remove `Builder`.

    Use `fasta::reader::Builder` and construct `IndexedReader` with a
    `fasta::Reader` instead.

## 0.13.0 - 2022-08-16

### Changed

  * fasta: Raise minimum supported Rust version (MSRV) to 1.57.0.

## 0.12.0 - 2022-07-05

### Changed

  * fasta: Sync dependencies.

## 0.11.0 - 2022-06-08

### Added

  * fasta/record/sequence: Implement `FromIterator<u8>`.

  * fasta/record/sequence: Add iterator to complement a sequence
    (`Sequence::complement`) ([#86]).

  * fasta/record/sequence: Add method to return a slice as a `Sequence`
    (`Sequence::slice`).

  * fasta/writer: Add builder ([#87]).

    This allows overriding the line base count.

[#86]: https://github.com/zaeleus/noodles/issues/86
[#87]: https://github.com/zaeleus/noodles/issues/87

### Changed

  * fasta/indexer: Sequence lines no longer strip end-of-line ASCII whitespace.

    This previously would take a line (i.e., characters up to `\n`) and strip
    [ASCII whitespace] from the end. To be consistent with the FASTA reader,
    all characters in the line are now considered bases. This leads to a ~20%
    performance improvement by avoiding having to copy the line buffer.

[ASCII whitespace]: https://infra.spec.whatwg.org/#ascii-whitespace

## 0.10.0 - 2022-04-14

### Added

  * fasta/repository/adapters/indexed_reader: Add convenience buidler for
    `BufReader<File>`.

## 0.9.0 - 2022-03-29

### Added

  * fasta/record/sequence: Add indexing by `Position`.

  * fasta/record/sequence: Implement `From<Bytes>`.

### Changed

  * fasta/repository: Box the adapter.

    `fasta::Repository` no longer carries an adapter generic.

  * fasta/repository: Implement `Default`.

  * fasta/repository/adapters: Add an empty adapter.

    This may be useful to create a repository that is never used.

## 0.8.0 - 2022-03-02

### Added

  * fasta: Add async reader (`fasta::AsyncReader`).

  * fasta: Add sequence repository (`fasta::Repository`).

    A repository is a concurrent cache that uses a storage adapter to lookup
    and load sequence data.

  * fasta/fai: Add async reader (`fai::AsyncReader`).

### Changed

  * fasta/record/sequence: `Sequence` is now backed by a `Bytes` buffer.

    This allows for zero-copy cloning of the sequence or slices of the
    sequence.

## 0.7.0 - 2022-02-17

### Added

  * fasta: Set minimum supported Rust version (MSRV) to 1.56.0 in package
    manifest.

## 0.6.0 - 2022-01-27

### Deprecated

  * fasta/record/definition: Deprecate
    `ParseError::MissingReferenceSequenceName`.

    Use `ParseError::MissingName` instead.

## 0.5.2 - 2022-01-13

### Fixed

  * fasta: Sync dependencies.

## 0.5.1 - 2021-12-09

### Fixed

  * fasta: Sync dependencies.

## 0.5.0 - 2021-12-02

### Added

  * fasta/record/definition: Accept `Into<String>` for name.

### Changed

  * fasta/record: Wrap sequence.

    Use `Sequence::as_ref` to get the underlying list.

## 0.4.1 - 2021-11-18

### Fixed

  * fasta: Sync dependencies.

## 0.4.0 - 2021-11-11

### Changed

  * fasta: Update to Rust 2021.

## 0.3.1 - 2021-10-16

### Fixed

  * fasta: Sync dependencies.

## 0.3.0 - 2021-10-01

### Deprecated

  * fasta/fai/record: `Record::reference_sequence_name` is now `Record::name`.

    FASTA records are not necessarily reference sequences.

## 0.2.4 - 2021-09-23

### Fixed

  * fasta: Sync dependencies.

## 0.2.3 - 2021-09-19

### Fixed

  * fasta: Sync dependencies.

## 0.2.2 - 2021-08-19

### Fixed

  * fasta: Sync dependencies.

## 0.2.1 - 2021-08-11

### Fixed

  * fasta: Sync dependencies.

## 0.2.0 - 2021-07-30

### Changed

  * fasta/record: Rename `reference_sequence_name` to `name`.

    FASTA records are not necessarily reference sequences.

## 0.1.1 - 2021-07-21

### Fixed

  * fasta: Fixed documentation link in package manifest ([#31]).

[#31]: https://github.com/zaeleus/noodles/issues/31

## 0.1.0 - 2021-07-14

  * fasta: Initial release.
