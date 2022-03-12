//! CRAM reader and record iterator.

mod container;
pub(crate) mod data_container;
pub(crate) mod num;
pub(crate) mod record;
mod records;

pub use self::records::Records;

use std::{
    io::{self, Read, Seek, SeekFrom},
    str,
};

use byteorder::{LittleEndian, ReadBytesExt};
use noodles_fasta as fasta;
use noodles_sam as sam;

use self::container::read_container;
use super::{container::Block, file_definition::Version, FileDefinition, MAGIC_NUMBER};
use crate::data_container::DataContainer;

/// A CRAM reader.
///
/// The CRAM format is comprised of four main parts: 1) a file definition, 2) a file header, 3) a
/// list of data containers, and 4) an end-of-file (EOF) container.
///
/// # Examples
///
/// ```no_run
/// # use std::{fs::File, io};
/// use noodles_cram as cram;
/// use noodles_fasta as fasta;
///
/// let repository = fasta::Repository::default();
///
/// let mut reader = File::open("sample.cram").map(cram::Reader::new)?;
/// reader.read_file_definition()?;
///
/// let header = reader.read_file_header()?.parse()?;
///
/// for result in reader.records(&repository, &header) {
///     let record = result?;
///     println!("{:?}", record);
/// }
///
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct Reader<R>
where
    R: Read,
{
    inner: R,
}

impl<R> Reader<R>
where
    R: Read,
{
    /// Creates a CRAM reader.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_cram as cram;
    /// let mut reader = File::open("sample.bam").map(cram::Reader::new)?;
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn new(reader: R) -> Self {
        Self { inner: reader }
    }

    /// Returns a reference to the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_cram as cram;
    /// let data = [];
    /// let reader = cram::Reader::new(&data[..]);
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
    /// use noodles_cram as cram;
    /// let data = [];
    /// let mut reader = cram::Reader::new(&data[..]);
    /// assert!(reader.get_mut().is_empty());
    /// ```
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Unwraps and returns the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_cram as cram;
    /// let data = [];
    /// let reader = cram::Reader::new(&data[..]);
    /// assert!(reader.into_inner().is_empty());
    /// ```
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Reads the CRAM file definition.
    ///
    /// The CRAM magic number is also checked.
    ///
    /// The position of the stream is expected to be at the start.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_cram as cram;
    /// let mut reader = File::open("sample.cram").map(cram::Reader::new)?;
    /// let file_definition = reader.read_file_definition()?;
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn read_file_definition(&mut self) -> io::Result<FileDefinition> {
        read_magic_number(&mut self.inner)?;

        let format = read_format(&mut self.inner)?;
        let file_id = read_file_id(&mut self.inner)?;

        Ok(FileDefinition::new(format, file_id))
    }

    /// Reads the raw SAM header.
    ///
    /// The position of the stream is expected to be at the CRAM header container, i.e., directly
    /// after the file definition.
    ///
    /// This returns the raw SAM header as a [`String`]. It can subsequently be parsed as a
    /// [`noodles_sam::Header`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_cram as cram;
    ///
    /// let mut reader = File::open("sample.cram").map(cram::Reader::new)?;
    /// reader.read_file_definition()?;
    ///
    /// let header = reader.read_file_header()?;
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn read_file_header(&mut self) -> io::Result<String> {
        let container = read_container(&mut self.inner)?;

        if let Some(block) = container.blocks().first() {
            read_file_header_block(block)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid header container: missing block for SAM header",
            ))
        }
    }

    pub(crate) fn read_data_container_with_container_header(
        &mut self,
    ) -> io::Result<Option<(crate::container::Header, DataContainer)>> {
        use self::data_container::read_data_container_with_container_header;
        read_data_container_with_container_header(&mut self.inner)
    }

    /// Reads a data container.
    ///
    /// This returns `None` if the container header is the EOF container header, which signals the
    /// end of the stream.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_cram as cram;
    ///
    /// let mut reader = File::open("sample.cram").map(cram::Reader::new)?;
    /// reader.read_file_definition()?;
    /// reader.read_file_header()?;
    ///
    /// while let Some(container) = reader.read_data_container()? {
    ///     // ...
    /// }
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn read_data_container(&mut self) -> io::Result<Option<DataContainer>> {
        use self::data_container::read_data_container;

        read_data_container(&mut self.inner)
    }

    /// Returns a iterator over records starting from the current stream position.
    ///
    /// The stream is expected to be at the start of a data container.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use noodles_cram as cram;
    /// use noodles_fasta as fasta;
    ///
    /// let repository = fasta::Repository::default();
    ///
    /// let mut reader = File::open("sample.cram").map(cram::Reader::new)?;
    /// reader.read_file_definition()?;
    ///
    /// let header = reader.read_file_header()?.parse()?;
    ///
    /// for result in reader.records(&repository, &header) {
    ///     let record = result?;
    ///     println!("{:?}", record);
    /// }
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn records<'a>(
        &'a mut self,
        reference_sequence_repository: &'a fasta::Repository,
        header: &'a sam::Header,
    ) -> Records<'a, R> {
        Records::new(self, reference_sequence_repository, header)
    }
}

impl<R> Reader<R>
where
    R: Read + Seek,
{
    /// Seeks the underlying reader to the given position.
    ///
    /// Positions typically come from the associated CRAM index file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::{fs::File, io};
    /// use std::io::SeekFrom;
    /// use noodles_cram as cram;
    ///
    /// let mut reader = File::open("sample.cram").map(cram::Reader::new)?;
    /// reader.seek(SeekFrom::Start(17711))?;
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }

    /// Returns the current position of the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::{self, Cursor};
    /// use noodles_cram as cram;
    /// let data = Cursor::new(Vec::new());
    /// let mut reader = cram::Reader::new(data);
    /// let position = reader.position()?;
    /// assert_eq!(position, 0);
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn position(&mut self) -> io::Result<u64> {
        self.inner.seek(SeekFrom::Current(0))
    }
}

impl<R> sam::AlignmentReader for Reader<R>
where
    R: Read,
{
    fn read_alignment_header(&mut self) -> io::Result<sam::Header> {
        self.read_file_definition()?;

        self.read_file_header().and_then(|s| {
            s.parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        })
    }

    fn alignment_records<'a>(
        &'a mut self,
        reference_sequence_repository: &'a fasta::Repository,
        header: &'a sam::Header,
    ) -> Box<dyn Iterator<Item = io::Result<sam::Record>> + 'a> {
        Box::new(
            self.records(reference_sequence_repository, header)
                .map(|result| result.and_then(|record| record.try_into_sam_record(header))),
        )
    }
}

fn read_magic_number<R>(reader: &mut R) -> io::Result<()>
where
    R: Read,
{
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;

    if buf == MAGIC_NUMBER {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid CRAM header",
        ))
    }
}

fn read_format<R>(reader: &mut R) -> io::Result<Version>
where
    R: Read,
{
    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;
    Ok(Version::new(buf[0], buf[1]))
}

fn read_file_id<R>(reader: &mut R) -> io::Result<[u8; 20]>
where
    R: Read,
{
    let mut buf = [0; 20];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

pub(crate) fn read_file_header_block(block: &Block) -> io::Result<String> {
    use crate::container::block::ContentType;

    if block.content_type() != ContentType::FileHeader {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid block content type: expected {:?}, got {:?}",
                ContentType::FileHeader,
                block.content_type()
            ),
        ));
    }

    let data = block.decompressed_data()?;
    let mut reader = &data[..];

    let _header_len = reader.read_i32::<LittleEndian>()?;

    str::from_utf8(reader)
        .map(|s| s.into())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use crate::container::block::ContentType;

    use super::*;

    #[test]
    fn test_read_file_definition() -> Result<(), Box<dyn std::error::Error>> {
        let data = [
            0x43, 0x52, 0x41, 0x4d, // magic number = b"CRAM"
            0x03, 0x00, // format version = (3, 0)
            0x00, 0x68, 0xac, 0xf3, 0x06, 0x4d, 0xaa, 0x1e, 0x29, 0xa4, 0xa0, 0x8c, 0x56, 0xee,
            0x91, 0x9b, 0x91, 0x04, 0x21, 0x1f, // file ID
        ];

        let mut reader = Reader::new(&data[..]);
        let actual = reader.read_file_definition()?;

        let file_id = <[u8; 20]>::try_from(&data[6..])?;
        let expected = FileDefinition::new(Version::new(3, 0), file_id);

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_read_file_header_block() -> io::Result<()> {
        let expected = "noodles";

        let header_data = expected.as_bytes();
        let header_data_len = header_data.len() as i32;

        let mut data = header_data_len.to_le_bytes().to_vec();
        data.extend(header_data);

        let block = Block::builder()
            .set_content_type(ContentType::FileHeader)
            .set_uncompressed_len(data.len())
            .set_data(data.to_vec())
            .build();

        let actual = read_file_header_block(&block)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_read_file_header_block_with_invalid_content_type() {
        let block = Block::builder()
            .set_content_type(ContentType::ExternalData)
            .build();

        assert!(matches!(
            read_file_header_block(&block),
            Err(ref e) if e.kind() == io::ErrorKind::InvalidData
        ));
    }

    #[test]
    fn test_read_magic_number() {
        let data = b"CRAM";
        let mut reader = &data[..];
        assert!(read_magic_number(&mut reader).is_ok());
    }

    #[test]
    fn test_read_magic_number_with_invalid_input() {
        let data = [];
        let mut reader = &data[..];
        assert!(matches!(
            read_magic_number(&mut reader),
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof,
        ));

        let data = b"BAM\x01";
        let mut reader = &data[..];
        assert!(matches!(
            read_magic_number(&mut reader),
            Err(ref e) if e.kind() == io::ErrorKind::InvalidData,
        ));
    }
}
