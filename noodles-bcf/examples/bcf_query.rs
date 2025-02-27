//! Queries a BCF file with a given region.
//!
//! The input BCF must have an index in the same directory.
//!
//! The result matches the output of `bcftools view --no-header <src> <region>`.

use std::{
    env,
    fs::File,
    io::{self, BufWriter},
    path::PathBuf,
};

use noodles_bcf as bcf;
use noodles_csi as csi;
use noodles_vcf as vcf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();

    let src = args.nth(1).map(PathBuf::from).expect("missing src");
    let raw_region = args.next().expect("missing region");

    let mut reader = File::open(&src).map(bcf::Reader::new)?;
    let header = reader.read_header()?;

    let index = csi::read(src.with_extension("bcf.csi"))?;

    let region = raw_region.parse()?;
    let query = reader.query(&header, &index, &region)?;

    let stdout = io::stdout().lock();
    let mut writer = vcf::Writer::new(BufWriter::new(stdout));

    for result in query {
        let record = result?;
        writer.write_record(&header, &record)?;
    }

    Ok(())
}
