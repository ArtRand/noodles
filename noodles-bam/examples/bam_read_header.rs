//! Prints the header of a BAM file.
//!
//! A BAM file header is a SAM header.
//!
//! The result matches the output of `samtools head <src>`.

use std::{env, io};

use noodles_bam as bam;

fn main() -> io::Result<()> {
    let src = env::args().nth(1).expect("missing src");

    let mut reader = bam::reader::Builder::default().build_from_path(src)?;
    let header = reader.read_header()?;
    print!("{header}");

    Ok(())
}
