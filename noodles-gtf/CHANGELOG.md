# Changelog

## 0.11.0 - 2023-06-01

### Changed

  * gtf: Sync dependencies.

## 0.10.0 - 2023-05-18

### Added

  * gtf/reader: Add query iterator (`Reader::query`) ([#158]).

[#158]: https://github.com/zaeleus/noodles/issues/158

## 0.9.0 - 2023-05-11

### Changed

  * gtf/record/attributes: Relax parser by allowing whitespace between entries
    and no final entry terminator ([#169]).

    The entry terminator (`;`) is now considered a separator rather than a
    terminator.

  * gtf/record/attributes/entry: The text representation of an entry no
    longer has a terminator (`;`).

[#169]: https://github.com/zaeleus/noodles/issues/169

## 0.8.0 - 2023-03-03

### Changed

  * gtf: Sync dependencies.

## 0.7.0 - 2023-02-03

### Added

  * gtf: Implement `std::error::Error::source` for errors.

### Changed

  * gtf: Raise minimum supported Rust version (MSRV) to 1.64.0.

## 0.6.0 - 2022-11-18

### Added

  * gtf/writer: Add line writer (`Writer::write_line`).

## 0.5.0 - 2022-10-20

### Changed

  * gtf: Sync dependencies.

## 0.4.0 - 2022-08-16

### Changed

  * gtf: Raise minimum supported Rust version (MSRV) to 1.57.0.

## 0.3.1 - 2022-06-08

### Fixed

  * gtf: Sync dependencies.

## 0.3.0 - 2022-03-29

### Changed

  * gff/record: Change start and end positions to `Position`.

## 0.2.0 - 2022-02-17

### Added

  * gtf: Set minimum supported Rust version (MSRV) to 1.56.0 in package
    manifest.

## 0.1.0 - 2021-11-11

  * gtf: Initial release.
