//! BCF reader and iterators.

mod header;
pub(crate) mod lazy_record;
pub(crate) mod query;
pub(crate) mod record;
mod records;
pub(crate) mod string_map;
pub(crate) mod value;

pub use self::{query::Query, records::Records};

use std::{
    io::{self, BufRead, Read, Seek},
    iter,
};

use byteorder::ReadBytesExt;
use noodles_bgzf as bgzf;
use noodles_core::Region;
use noodles_csi as csi;
use noodles_vcf as vcf;

use self::{header::read_header, lazy_record::read_lazy_record, record::read_record};
use super::lazy;
use crate::header::string_maps::{ContigStringMap, StringMaps};

/// A BCF reader.
///
/// The BCF format is comprised of two parts: 1) a VCF header and 2) a list of records.
pub struct Reader<R> {
    inner: R,
    buf: Vec<u8>,
    string_maps: StringMaps,
}

impl<R> Reader<R>
where
    R: Read,
{
    /// Returns a reference to the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf as bcf;
    /// let data = [];
    /// let reader = bcf::Reader::from(&data[..]);
    /// assert!(reader.get_ref().is_empty());
    /// ```
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Returns a mutable reference to the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf as bcf;
    /// let data = [];
    /// let mut reader = bcf::Reader::from(&data[..]);
    /// assert!(reader.get_mut().is_empty());
    /// ```
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Returns the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf as bcf;
    /// let data = [];
    /// let reader = bcf::Reader::from(&data[..]);
    /// assert!(reader.into_inner().is_empty());
    /// ```
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Returns the string maps.
    ///
    /// This is only built after reading the header using [`Self::read_header`].
    pub fn string_maps(&self) -> &StringMaps {
        &self.string_maps
    }

    /// Reads the VCF header.
    ///
    /// The BCF magic number is checked, and the file format version is discarded.
    ///
    /// The position of the stream is expected to be at the start.
    ///
    /// This returns both the parsed VCF header and the associated string maps built from the raw
    /// header.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_bcf as bcf;
    /// let mut reader = File::open("sample.bcf").map(bcf::Reader::new)?;
    /// let header = reader.read_header()?;
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn read_header(&mut self) -> io::Result<vcf::Header> {
        read_magic(&mut self.inner)?;
        read_format_version(&mut self.inner)?;
        let (header, string_maps) = read_header(&mut self.inner)?;
        self.string_maps = string_maps;
        Ok(header)
    }

    /// Reads a single record.
    ///
    /// The stream is expected to be directly after the header or at the start of another record.
    ///
    /// It is more ergonomic to read records using an iterator (see [`Self::records`]), but using
    /// this method directly allows the reuse of a single [`vcf::Record`] buffer.
    ///
    /// If successful, the record size is returned. If a record size of 0 is returned, the stream
    /// reached EOF.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_bcf as bcf;
    /// use noodles_vcf as vcf;
    ///
    /// let mut reader = File::open("sample.bcf").map(bcf::Reader::new)?;
    /// let header = reader.read_header()?;
    ///
    /// let mut record = vcf::Record::default();
    /// reader.read_record(&header, &mut record)?;
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn read_record(
        &mut self,
        header: &vcf::Header,
        record: &mut vcf::Record,
    ) -> io::Result<usize> {
        read_record(
            &mut self.inner,
            header,
            &self.string_maps,
            &mut self.buf,
            record,
        )
    }

    /// Reads a single record without eagerly decoding (most of) its fields.
    ///
    /// The stream is expected to be directly after the header or at the start of another record.
    ///
    /// It is more ergnomic to read records using an iterator (see [`Self::lazy_records`]), but
    /// using this method directly allows the reuse of a single [`lazy::Record`] buffer.
    ///
    /// If successful, the record size is returned. If a record size of 0 is returned, the stream
    /// reached EOF.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_bcf as bcf;
    ///
    /// let mut reader = File::open("sample.bcf").map(bcf::Reader::new)?;
    /// reader.read_header()?;
    ///
    /// let mut record = bcf::lazy::Record::default();
    /// reader.read_lazy_record(&mut record)?;
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn read_lazy_record(&mut self, record: &mut lazy::Record) -> io::Result<usize> {
        read_lazy_record(&mut self.inner, &mut self.buf, record)
    }

    /// Returns an iterator over records starting from the current stream position.
    ///
    /// The stream is expected to be directly after the reference sequences or at the start of
    /// another record.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_bcf as bcf;
    ///
    /// let mut reader = File::open("sample.bcf").map(bcf::Reader::new)?;
    /// let header = reader.read_header()?;
    ///
    /// for result in reader.records(&header) {
    ///     let record = result?;
    ///     // ...
    /// }
    /// # Ok::<(), io::Error>(())
    pub fn records<'r, 'h>(&'r mut self, header: &'h vcf::Header) -> Records<'r, 'h, R> {
        Records::new(self, header)
    }

    /// Returns an iterator over lazy records starting from the current stream position.
    ///
    /// The stream is expected to be directly after the header or at the start of another record.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_bcf as bcf;
    ///
    /// let mut reader = File::open("sample.bcf").map(bcf::Reader::new)?;
    /// reader.read_header()?;
    ///
    /// for result in reader.lazy_records() {
    ///     let record = result?;
    ///     println!("{:?}", record);
    /// }
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn lazy_records(&mut self) -> impl Iterator<Item = io::Result<lazy::Record>> + '_ {
        let mut record = lazy::Record::default();

        iter::from_fn(move || match self.read_lazy_record(&mut record) {
            Ok(0) => None,
            Ok(_) => Some(Ok(record.clone())),
            Err(e) => Some(Err(e)),
        })
    }
}

impl<R> Reader<bgzf::Reader<R>>
where
    R: Read,
{
    /// Creates a BCF reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf as bcf;
    /// let data = [];
    /// let reader = bcf::Reader::new(&data[..]);
    /// ```
    pub fn new(reader: R) -> Self {
        Self::from(bgzf::Reader::new(reader))
    }

    /// Returns the current virtual position of the underlying BGZF reader.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bcf as bcf;
    ///
    /// let data = Vec::new();
    /// let reader = bcf::Reader::new(&data[..]);
    /// let virtual_position = reader.virtual_position();
    ///
    /// assert_eq!(virtual_position.compressed(), 0);
    /// assert_eq!(virtual_position.uncompressed(), 0);
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn virtual_position(&self) -> bgzf::VirtualPosition {
        self.inner.virtual_position()
    }
}

impl<R> Reader<bgzf::Reader<R>>
where
    R: Read + Seek,
{
    /// Seeks the underlying BGZF reader to the given virtual position.
    ///
    /// Virtual positions typically come from an associated BCF index file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_bcf as bcf;
    /// use noodles_bgzf as bgzf;
    ///
    /// let mut reader = File::open("sample.bcf").map(bcf::Reader::new)?;
    ///
    /// let virtual_position = bgzf::VirtualPosition::from(102334155);
    /// reader.seek(virtual_position)?;
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn seek(&mut self, pos: bgzf::VirtualPosition) -> io::Result<bgzf::VirtualPosition> {
        self.inner.seek(pos)
    }

    /// Returns an iterator over records that intersects the given region.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::fs::File;
    /// use noodles_bcf::{self as bcf, header::StringMaps};
    /// use noodles_core::Region;
    /// use noodles_csi as csi;
    ///
    /// let mut reader = File::open("sample.bcf").map(bcf::Reader::new)?;
    /// let header = reader.read_header()?;
    ///
    /// let index = csi::read("sample.bcf.csi")?;
    /// let region = "sq0:8-13".parse()?;
    /// let query = reader.query(&header, &index, &region)?;
    ///
    /// for result in query {
    ///     let record = result?;
    ///     // ...
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn query<'r, 'h>(
        &'r mut self,
        header: &'h vcf::Header,
        index: &csi::Index,
        region: &Region,
    ) -> io::Result<Query<'r, 'h, R>> {
        let reference_sequence_id = resolve_region(self.string_maps.contigs(), region)?;
        let chunks = index.query(reference_sequence_id, region.interval())?;

        Ok(Query::new(
            self.get_mut(),
            header,
            chunks,
            reference_sequence_id,
            region.interval(),
        ))
    }
}

impl<R> From<R> for Reader<R> {
    fn from(inner: R) -> Self {
        Self {
            inner,
            buf: Vec::new(),
            string_maps: StringMaps::default(),
        }
    }
}

impl<R> vcf::VariantReader<R> for Reader<R>
where
    R: BufRead,
{
    fn read_variant_header(&mut self) -> io::Result<vcf::Header> {
        self.read_header()
    }

    fn variant_records<'r, 'h: 'r>(
        &'r mut self,
        header: &'h vcf::Header,
    ) -> Box<dyn Iterator<Item = io::Result<vcf::Record>> + 'r> {
        Box::new(self.records(header))
    }
}

fn read_magic<R>(reader: &mut R) -> io::Result<()>
where
    R: Read,
{
    use crate::MAGIC_NUMBER;

    let mut buf = [0; 3];
    reader.read_exact(&mut buf)?;

    if buf == MAGIC_NUMBER {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid BCF header",
        ))
    }
}

fn read_format_version<R>(reader: &mut R) -> io::Result<(u8, u8)>
where
    R: Read,
{
    let major_version = reader.read_u8()?;
    let minor_version = reader.read_u8()?;

    Ok((major_version, minor_version))
}

pub(crate) fn resolve_region(
    contig_string_map: &ContigStringMap,
    region: &Region,
) -> io::Result<usize> {
    contig_string_map
        .get_index_of(region.name())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("region does not exist in contigs: {region:?}"),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_magic() {
        let data = b"BCF";
        let mut reader = &data[..];
        assert!(read_magic(&mut reader).is_ok());

        let data = [];
        let mut reader = &data[..];
        assert!(matches!(
            read_magic(&mut reader),
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof
        ));

        let data = b"BAM";
        let mut reader = &data[..];
        assert!(matches!(
            read_magic(&mut reader),
            Err(ref e) if e.kind() == io::ErrorKind::InvalidData
        ));
    }

    #[test]
    fn test_read_format_version() -> io::Result<()> {
        let data = [0x02, 0x01];
        let mut reader = &data[..];
        assert_eq!(read_format_version(&mut reader)?, (2, 1));
        Ok(())
    }
}
