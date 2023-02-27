//! BAM record data field component writers.

mod tag;
mod value;

use std::io;

use bytes::BufMut;
use noodles_sam::record::{
    data::field::{
        value::{Subtype, Type},
        Tag, Value,
    },
    Cigar,
};

use self::tag::put_tag;
pub use self::value::put_value;

pub(super) fn put_field<B>(dst: &mut B, tag: Tag, value: &Value) -> io::Result<()>
where
    B: BufMut,
{
    put_tag(dst, tag);
    value::put_type(dst, value.ty());
    put_value(dst, value)?;
    Ok(())
}

pub(crate) fn put_cigar<B>(dst: &mut B, cigar: &Cigar) -> io::Result<()>
where
    B: BufMut,
{
    put_tag(dst, Tag::Cigar);
    value::put_type(dst, Type::Array);
    value::put_array_header(dst, Subtype::UInt32, cigar.len())?;
    crate::writer::record::put_cigar(dst, cigar)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_field() -> io::Result<()> {
        let mut buf = Vec::new();
        let (tag, value) = (Tag::AlignmentHitCount, Value::from(1));
        put_field(&mut buf, tag, &value)?;
        assert_eq!(buf, [b'N', b'H', b'C', 0x01]);
        Ok(())
    }

    #[test]
    fn test_put_cigar() -> Result<(), Box<dyn std::error::Error>> {
        use noodles_sam::record::cigar::{op::Kind, Op};

        let mut buf = Vec::new();
        let cigar = Cigar::try_from(vec![Op::new(Kind::Match, 4)])?;
        put_cigar(&mut buf, &cigar)?;

        let expected = [
            b'C', b'G', // tag = CG
            b'B', b'I', // type = [u32]
            0x01, 0x00, 0x00, 0x00, // count = 1
            0x40, 0x00, 0x00, 0x00, // cigar = 4M
        ];

        assert_eq!(buf, expected);

        Ok(())
    }
}
