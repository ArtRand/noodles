use std::io::{self, Write};

use super::MISSING;
use crate::record::{
    genotypes::{values::field::Value, Keys, Values},
    Genotypes,
};

pub(super) fn write_genotypes<W>(writer: &mut W, genotypes: &Genotypes) -> io::Result<()>
where
    W: Write,
{
    const DELIMITER: &[u8] = b"\t";

    write_keys(writer, genotypes.keys())?;

    for values in genotypes.iter() {
        writer.write_all(DELIMITER)?;
        write_values(writer, values)?;
    }

    Ok(())
}

fn write_keys<W>(writer: &mut W, keys: &Keys) -> io::Result<()>
where
    W: Write,
{
    const DELIMITER: &[u8] = b":";

    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            writer.write_all(DELIMITER)?;
        }

        writer.write_all(key.as_ref().as_bytes())?;
    }

    Ok(())
}

fn write_values<W>(writer: &mut W, values: &Values) -> io::Result<()>
where
    W: Write,
{
    const DELIMITER: &[u8] = b":";

    for (i, (_, value)) in values.iter().enumerate() {
        if i > 0 {
            writer.write_all(DELIMITER)?;
        }

        match value {
            Some(v) => write_value(writer, v)?,
            None => writer.write_all(MISSING)?,
        }
    }

    Ok(())
}

fn write_value<W>(writer: &mut W, value: &Value) -> io::Result<()>
where
    W: Write,
{
    const DELIMITER: &[u8] = b",";

    match value {
        Value::Integer(n) => write!(writer, "{n}"),
        Value::Float(n) => write!(writer, "{n}"),
        Value::Character(c) => write!(writer, "{c}"),
        Value::String(s) => writer.write_all(s.as_bytes()),
        Value::IntegerArray(values) => {
            for (i, v) in values.iter().enumerate() {
                if i > 0 {
                    writer.write_all(DELIMITER)?;
                }

                if let Some(n) = v {
                    write!(writer, "{n}")?;
                } else {
                    writer.write_all(MISSING)?;
                }
            }

            Ok(())
        }
        Value::FloatArray(values) => {
            for (i, v) in values.iter().enumerate() {
                if i > 0 {
                    writer.write_all(DELIMITER)?;
                }

                if let Some(n) = v {
                    write!(writer, "{n}")?;
                } else {
                    writer.write_all(MISSING)?;
                }
            }

            Ok(())
        }
        Value::CharacterArray(values) => {
            for (i, v) in values.iter().enumerate() {
                if i > 0 {
                    writer.write_all(DELIMITER)?;
                }

                if let Some(c) = v {
                    write!(writer, "{c}")?;
                } else {
                    writer.write_all(MISSING)?;
                }
            }

            Ok(())
        }
        Value::StringArray(values) => {
            for (i, v) in values.iter().enumerate() {
                if i > 0 {
                    writer.write_all(DELIMITER)?;
                }

                if let Some(s) = v {
                    writer.write_all(s.as_bytes())?;
                } else {
                    writer.write_all(MISSING)?;
                }
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_genotypes() -> Result<(), Box<dyn std::error::Error>> {
        use crate::header::format::key;

        fn t(buf: &mut Vec<u8>, genotypes: &Genotypes, expected: &[u8]) -> io::Result<()> {
            buf.clear();
            write_genotypes(buf, genotypes)?;
            assert_eq!(buf, expected);
            Ok(())
        }

        let mut buf = Vec::new();

        let genotypes = Genotypes::new(
            Keys::try_from(vec![key::GENOTYPE])?,
            vec![[(key::GENOTYPE, Some(Value::String(String::from("0|0"))))]
                .into_iter()
                .collect()],
        );
        t(&mut buf, &genotypes, b"GT\t0|0")?;

        let genotypes = Genotypes::new(
            Keys::try_from(vec![key::GENOTYPE, key::CONDITIONAL_GENOTYPE_QUALITY])?,
            vec![
                [
                    (key::GENOTYPE, Some(Value::String(String::from("0|0")))),
                    (key::CONDITIONAL_GENOTYPE_QUALITY, Some(Value::Integer(13))),
                ]
                .into_iter()
                .collect(),
                [
                    (key::GENOTYPE, Some(Value::String(String::from("0/1")))),
                    (key::CONDITIONAL_GENOTYPE_QUALITY, Some(Value::Integer(8))),
                ]
                .into_iter()
                .collect(),
            ],
        );
        t(&mut buf, &genotypes, b"GT:GQ\t0|0:13\t0/1:8")?;

        Ok(())
    }

    #[test]
    fn test_write_value() -> io::Result<()> {
        fn t(buf: &mut Vec<u8>, value: &Value, expected: &[u8]) -> io::Result<()> {
            buf.clear();
            write_value(buf, value)?;
            assert_eq!(buf, expected);
            Ok(())
        }

        let mut buf = Vec::new();

        t(&mut buf, &Value::Integer(8), b"8")?;
        t(&mut buf, &Value::Float(0.333), b"0.333")?;
        t(&mut buf, &Value::Character('n'), b"n")?;
        t(
            &mut buf,
            &Value::String(String::from("noodles")),
            b"noodles",
        )?;

        t(&mut buf, &Value::IntegerArray(vec![Some(8)]), b"8")?;
        t(
            &mut buf,
            &Value::IntegerArray(vec![Some(8), Some(13)]),
            b"8,13",
        )?;
        t(&mut buf, &Value::IntegerArray(vec![Some(8), None]), b"8,.")?;

        t(&mut buf, &Value::FloatArray(vec![Some(0.333)]), b"0.333")?;
        t(
            &mut buf,
            &Value::FloatArray(vec![Some(0.333), Some(0.667)]),
            b"0.333,0.667",
        )?;
        t(
            &mut buf,
            &Value::FloatArray(vec![Some(0.333), None]),
            b"0.333,.",
        )?;

        t(&mut buf, &Value::CharacterArray(vec![Some('n')]), b"n")?;
        t(
            &mut buf,
            &Value::CharacterArray(vec![Some('n'), Some('d')]),
            b"n,d",
        )?;
        t(
            &mut buf,
            &Value::CharacterArray(vec![Some('n'), None]),
            b"n,.",
        )?;

        t(
            &mut buf,
            &Value::StringArray(vec![Some(String::from("noodles"))]),
            b"noodles",
        )?;
        t(
            &mut buf,
            &Value::StringArray(vec![
                Some(String::from("noodles")),
                Some(String::from("vcf")),
            ]),
            b"noodles,vcf",
        )?;
        t(
            &mut buf,
            &Value::StringArray(vec![Some(String::from("noodles")), None]),
            b"noodles,.",
        )?;

        Ok(())
    }
}
