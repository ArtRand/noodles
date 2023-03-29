use std::{io, mem};

use noodles_core::Position;

use super::{
    reference_sequence::{self, bin::Chunk},
    Index, ReferenceSequence,
};

/// A CSI indexer.
#[derive(Debug, Default)]
pub struct Indexer {
    min_shift: u8,
    depth: u8,
    reference_sequence_builder: reference_sequence::Builder,
    reference_sequences: Vec<ReferenceSequence>,
    unplaced_unmapped_record_count: u64,
}

impl Indexer {
    /// Creates a CSI indexer.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::index::Indexer;
    /// let indexer = Indexer::new(14, 5);
    /// ```
    pub fn new(min_shift: u8, depth: u8) -> Self {
        Self {
            min_shift,
            depth,
            reference_sequence_builder: reference_sequence::Builder::default(),
            reference_sequences: Vec::new(),
            unplaced_unmapped_record_count: 0,
        }
    }

    /// Adds a record.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;
    /// use noodles_core::Position;
    /// use noodles_csi::index::{reference_sequence::bin::Chunk, Indexer};
    ///
    /// let mut indexer = Indexer::new(14, 5);
    ///
    /// let reference_sequence_id = 0;
    /// let start = Position::try_from(8)?;
    /// let end = Position::try_from(13)?;
    /// let is_mapped = true;
    /// let chunk = Chunk::new(
    ///     bgzf::VirtualPosition::from(144),
    ///     bgzf::VirtualPosition::from(233),
    /// );
    ///
    /// indexer.add_record(Some((reference_sequence_id, start, end, is_mapped)), chunk)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_record(
        &mut self,
        alignment_context: Option<(usize, Position, Position, bool)>,
        chunk: Chunk,
    ) -> io::Result<()> {
        use std::cmp::Ordering;

        let (reference_sequence_id, start, end, is_mapped) = if let Some(ctx) = alignment_context {
            ctx
        } else {
            self.unplaced_unmapped_record_count += 1;
            return Ok(());
        };

        match reference_sequence_id.cmp(&self.current_reference_sequence_id()) {
            Ordering::Less => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid reference sequence ID",
                ));
            }
            Ordering::Equal => {}
            Ordering::Greater => self.add_reference_sequences_builders_until(reference_sequence_id),
        }

        self.reference_sequence_builder.add_record(
            self.min_shift,
            self.depth,
            start,
            end,
            is_mapped,
            chunk,
        );

        Ok(())
    }

    /// Builds a CSI index.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::index::Indexer;
    /// let indexer = Indexer::new(14, 5);
    /// let index = indexer.build(0);
    /// ```
    pub fn build(mut self, reference_sequence_count: usize) -> Index {
        if reference_sequence_count == 0 {
            return Index::builder()
                .set_unplaced_unmapped_record_count(self.unplaced_unmapped_record_count)
                .build();
        }

        // SAFETY: `reference_sequence_count` is > 0.
        let last_reference_sequence_id = reference_sequence_count - 1;
        self.add_reference_sequences_builders_until(last_reference_sequence_id);

        Index::builder()
            .set_reference_sequences(self.reference_sequences)
            .set_unplaced_unmapped_record_count(self.unplaced_unmapped_record_count)
            .build()
    }

    fn current_reference_sequence_id(&self) -> usize {
        self.reference_sequences.len()
    }

    fn add_reference_sequences_builders_until(&mut self, reference_sequence_id: usize) {
        while self.reference_sequences.len() < reference_sequence_id {
            let reference_sequence_builder = mem::take(&mut self.reference_sequence_builder);
            let reference_sequence = reference_sequence_builder.build();
            self.reference_sequences.push(reference_sequence);
        }
    }
}
