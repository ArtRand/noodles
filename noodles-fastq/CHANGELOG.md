# Changelog

## Unreleased

### Added

  * fastq/fai/record: Implement `std::error::Error::source` for `ParseError`.

### Changed

  * fastq: Raise minimum supported Rust version (MSRV) to 1.64.0.

### Removed

  * fastq: Remove `Record::name`.

    This was deprecated in noodles-fastq 0.2.0. Use `Record::name`
    instead.

## 0.5.1 - 2022-10-28

### Fixed

  * fastq/reader: Increase the visibility of the module (`reader`) ([#118]).

    This allows access to the `reader::Records` iterator.

[#118]: https://github.com/zaeleus/noodles/issues/118

## 0.5.0 - 2022-02-17

### Added

  * fastq: Set minimum supported Rust version (MSRV) to 1.56.0 in package
    manifest.

## 0.4.0 - 2022-01-27

### Added

   * fastq/record: Add description field (`Record::description`).

   * fastq/record: Add mutable getters for name (`Record::name_mut`),
     sequence (`Record::sequence_mut`), description
     (`Record::description_mut`), and quality scores
     (`Record::quality_scores_mut`).

### Changed

  * fastq/async/reader: Ensure the record description (line 3) is prefixed
    with a plus sign (`+`).

  * fastq/reader: Ensure the record description (line 3) is prefixed with a
    plus sign (`+`).

## 0.3.0 - 2021-11-11

### Added

  * fastq/reader: Add common methods to access the underlying reader:
    `get_ref`, `get_mut`, and `into_inner`.

### Changed

  * fastq: Update to Rust 2021.

## 0.2.0 - 2021-10-01

### Added

  * fastq/async: Add async reader (`fastq::AsyncReader`).

  * fastq/async: Add async writer (`fastq::AsyncWriter`).

    Async I/O can be enabled with the `async` feature.

### Deprecated

  * fastq/fai/record: `Record::read_name` is now `Record::name`.

  * fastq/record: `Record::read_name` is now `Record::name`.

    FASTQ record names are not necessarily read names.

## 0.1.1 - 2021-07-21

### Fixed

  * fastq: Fixed documentation link in package manifest ([#31]).

[#31]: https://github.com/zaeleus/noodles/issues/31

## 0.1.0 - 2021-07-14

  * fastq: Initial release.
