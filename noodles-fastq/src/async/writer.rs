use tokio::io::{self, AsyncWrite, AsyncWriteExt};

use crate::Record;

/// An async FASTQ writer.
pub struct Writer<W> {
    inner: W,
}

impl<W> Writer<W>
where
    W: AsyncWrite + Unpin,
{
    /// Creates an async FASTQ writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_fastq as fastq;
    /// let writer = fastq::AsyncWriter::new(Vec::new());
    /// ```
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    /// Returns a reference to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_fastq as fastq;
    /// let writer = fastq::AsyncWriter::new(Vec::new());
    /// assert!(writer.get_ref().is_empty());
    /// ```
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Returns the underlying writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_fastq as fastq;
    /// let writer = fastq::AsyncWriter::new(Vec::new());
    /// assert!(writer.into_inner().is_empty());
    /// ```
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// Writes a FASTQ record.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> std::io::Result<()> {
    /// use noodles_fastq as fastq;
    ///
    /// let mut writer = fastq::AsyncWriter::new(Vec::new());
    ///
    /// let record = fastq::Record::new("r0", "ATCG", "NDLS");
    /// writer.write_record(&record).await?;
    ///
    /// assert_eq!(writer.get_ref(), b"@r0\nATCG\n+\nNDLS\n");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write_record(&mut self, record: &Record) -> io::Result<()> {
        write_record(&mut self.inner, record).await
    }
}

async fn write_record<W>(writer: &mut W, record: &Record) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    writer.write_all(b"@").await?;
    writer.write_all(record.name()).await?;
    writer.write_all(b"\n").await?;

    writer.write_all(record.sequence()).await?;
    writer.write_all(b"\n+\n").await?;

    writer.write_all(record.quality_scores()).await?;
    writer.write_all(b"\n").await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_record() -> io::Result<()> {
        let record = Record::new("r0", "ACGT", "NDLS");

        let mut buf = Vec::new();
        write_record(&mut buf, &record).await?;

        let expected = b"@r0\nACGT\n+\nNDLS\n";
        assert_eq!(buf, expected);

        Ok(())
    }
}
