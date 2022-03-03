use std::{
    io::{self, Read},
    vec,
};

use noodles_fasta as fasta;
use noodles_sam as sam;

use super::Reader;
use crate::Record;

/// An iterator over records of a CRAM reader.
///
/// This is created by calling [`Reader::records`].
pub struct Records<'a, R>
where
    R: Read,
{
    reader: &'a mut Reader<R>,
    reference_sequence_repository: &'a fasta::Repository,
    header: &'a sam::Header,
    records: vec::IntoIter<Record>,
}

impl<'a, R> Records<'a, R>
where
    R: Read,
{
    pub(crate) fn new(
        reader: &'a mut Reader<R>,
        reference_sequence_repository: &'a fasta::Repository,
        header: &'a sam::Header,
    ) -> Self {
        Self {
            reader,
            reference_sequence_repository,
            header,
            records: Vec::new().into_iter(),
        }
    }

    fn read_container_records(&mut self) -> io::Result<bool> {
        let container = match self.reader.read_data_container()? {
            Some(c) => c,
            None => return Ok(true),
        };

        self.records = container
            .slices()
            .iter()
            .map(|slice| {
                let compression_header = container.compression_header();

                slice.records(compression_header).and_then(|mut records| {
                    slice.resolve_records(
                        self.reference_sequence_repository,
                        self.header,
                        compression_header,
                        &mut records,
                    )?;

                    Ok(records)
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .into_iter();

        Ok(false)
    }
}

impl<'a, R> Iterator for Records<'a, R>
where
    R: Read,
{
    type Item = io::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.records.next() {
                Some(r) => return Some(Ok(r)),
                None => match self.read_container_records() {
                    Ok(true) => return None,
                    Ok(false) => {}
                    Err(e) => return Some(Err(e)),
                },
            }
        }
    }
}
