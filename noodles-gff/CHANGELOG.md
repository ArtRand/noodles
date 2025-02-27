# Changelog

## 0.13.0 - 2023-06-01

### Changed

  * gff: Sync dependencies.

## 0.12.0 - 2023-05-18

### Added

  * gff/reader: Add query iterator (`Reader::query`) ([#158]).

[#158]: https://github.com/zaeleus/noodles/issues/158

## 0.11.0 - 2023-03-03

### Changed

  * gff: Sync dependencies.

## 0.10.0 - 2023-02-03

### Added

  * gff: Implement `std::error::Error::source` for errors.

### Changed

  * gff: Raise minimum supported Rust version (MSRV) to 1.64.0.

## 0.9.0 - 2022-10-28

### Added

  * gff/writer: Add line writer (`Writer::write_line`).

## 0.8.0 - 2022-10-20

### Changed

  * gff: Sync dependencies.

## 0.7.0 - 2022-08-16

### Changed

  * gff: Raise minimum supported Rust version (MSRV) to 1.57.0.

## 0.6.1 - 2022-06-08

### Fixed

  * gff: Sync dependencies.

## 0.6.0 - 2022-03-29

### Changed

  * gff/record: Change start and end positions to `Position`.

## 0.5.0 - 2022-02-17

### Added

  * gff: Set minimum supported Rust version (MSRV) to 1.56.0 in package
    manifest.

## 0.4.0 - 2021-11-11

### Changed

  * gff: Update to Rust 2021.

## 0.3.0 - 2021-10-16

### Added

  * gff/record: Implement `fmt::Display`.

    The string representation of a `Record` is its serialized tabular form.

### Changed

  * gff/record: Disallow reference sequence names to start with '>'.

## 0.2.0 - 2021-09-19

### Added

  * gff/record/attributes/entry: Accept `Into<String>` for key and value.

### Changed

  * gff/record/attributes/entry: Return `ParseError::Invalid` when no `=`
    separator is present.

    This previously would return `ParseError::MissingValue`.

## 0.1.1 - 2021-07-21

### Fixed

  * gff: Fixed documentation link in package manifest ([#31]).

[#31]: https://github.com/zaeleus/noodles/issues/31

## 0.1.0 - 2021-07-14

  * gff: Initial release.
