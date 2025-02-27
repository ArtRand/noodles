//! Counts the number of records in a BAM file.
//!
//! The result matches the output of `samtools view --count <src>`.

use std::env;

use noodles_bam as bam;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = env::args().nth(1).expect("missing src");

    let mut reader = bam::reader::Builder::default().build_from_path(src)?;
    let header = reader.read_header()?;

    let mut n = 0;

    for result in reader.records(&header) {
        let _ = result?;
        n += 1;
    }

    println!("{n}");

    Ok(())
}
