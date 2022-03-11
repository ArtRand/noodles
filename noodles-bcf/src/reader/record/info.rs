use std::io::{self, Read};

use noodles_vcf::{self as vcf, header::info::Type};

use crate::{
    header::string_maps::StringStringMap,
    reader::{string_map::read_string_map_index, value::read_value},
    record::{
        value::{Float, Int16, Int32, Int8},
        Value,
    },
};

pub fn read_info<R>(
    reader: &mut R,
    infos: &vcf::header::Infos,
    string_string_map: &StringStringMap,
    len: usize,
) -> io::Result<vcf::record::Info>
where
    R: Read,
{
    let mut fields = Vec::with_capacity(len);

    for _ in 0..len {
        let field = read_info_field(reader, infos, string_string_map)?;
        fields.push(field);
    }

    vcf::record::Info::try_from(fields).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn read_info_field<R>(
    reader: &mut R,
    infos: &vcf::header::Infos,
    string_string_map: &StringStringMap,
) -> io::Result<vcf::record::info::Field>
where
    R: Read,
{
    let key = read_info_field_key(reader, infos, string_string_map)?;

    let info = infos.get(&key).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("missing header INFO record for {}", key),
        )
    })?;

    let value = read_info_field_value(reader, info)?;

    Ok(vcf::record::info::Field::new(key, value))
}

fn read_info_field_key<R>(
    reader: &mut R,
    infos: &vcf::header::Infos,
    string_string_map: &StringStringMap,
) -> io::Result<vcf::header::info::Key>
where
    R: Read,
{
    read_string_map_index(reader)
        .and_then(|j| {
            string_string_map.get_index(j).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid string map index: {}", j),
                )
            })
        })
        .and_then(|raw_key| {
            infos
                .keys()
                .find(|k| k.as_ref() == raw_key)
                .cloned()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("missing header INFO record for {}", raw_key),
                    )
                })
        })
}

fn read_info_field_value<R>(
    reader: &mut R,
    info: &vcf::header::Info,
) -> io::Result<Option<vcf::record::info::field::Value>>
where
    R: Read,
{
    match info.ty() {
        Type::Integer => read_info_field_integer_value(reader),
        Type::Flag => read_info_field_flag_value(reader),
        Type::Float => read_info_field_float_value(reader),
        Type::Character => read_info_field_character_value(reader),
        Type::String => read_info_field_string_value(reader),
    }
}

fn read_info_field_integer_value<R>(
    reader: &mut R,
) -> io::Result<Option<vcf::record::info::field::Value>>
where
    R: Read,
{
    match read_value(reader)? {
        None
        | Some(Value::Int8(None | Some(Int8::Missing)))
        | Some(Value::Int16(None | Some(Int16::Missing)))
        | Some(Value::Int32(None | Some(Int32::Missing))) => Ok(None),
        Some(Value::Int8(Some(Int8::Value(n)))) => {
            Ok(Some(vcf::record::info::field::Value::Integer(i32::from(n))))
        }
        Some(Value::Int8Array(values)) => Ok(Some(vcf::record::info::field::Value::IntegerArray(
            values
                .into_iter()
                .map(Int8::from)
                .map(|value| match value {
                    Int8::Value(n) => Some(i32::from(n)),
                    Int8::Missing => None,
                    _ => todo!("unhandled i8 array value: {:?}", value),
                })
                .collect(),
        ))),
        Some(Value::Int16(Some(Int16::Value(n)))) => {
            Ok(Some(vcf::record::info::field::Value::Integer(i32::from(n))))
        }
        Some(Value::Int16Array(values)) => Ok(Some(vcf::record::info::field::Value::IntegerArray(
            values
                .into_iter()
                .map(Int16::from)
                .map(|value| match value {
                    Int16::Value(n) => Some(i32::from(n)),
                    Int16::Missing => None,
                    _ => todo!("unhandled i16 array value: {:?}", value),
                })
                .collect(),
        ))),
        Some(Value::Int32(Some(Int32::Value(n)))) => {
            Ok(Some(vcf::record::info::field::Value::Integer(n)))
        }
        Some(Value::Int32Array(values)) => Ok(Some(vcf::record::info::field::Value::IntegerArray(
            values
                .into_iter()
                .map(Int32::from)
                .map(|value| match value {
                    Int32::Value(n) => Some(n),
                    Int32::Missing => None,
                    _ => todo!("unhandled i32 array value: {:?}", value),
                })
                .collect(),
        ))),
        v => Err(type_mismatch_error(v, Type::Integer)),
    }
}

fn read_info_field_flag_value<R>(
    reader: &mut R,
) -> io::Result<Option<vcf::record::info::field::Value>>
where
    R: Read,
{
    match read_value(reader)? {
        None | Some(Value::Int8(Some(Int8::Value(1)))) => {
            Ok(Some(vcf::record::info::field::Value::Flag))
        }
        v => Err(type_mismatch_error(v, Type::Flag)),
    }
}

fn read_info_field_float_value<R>(
    reader: &mut R,
) -> io::Result<Option<vcf::record::info::field::Value>>
where
    R: Read,
{
    match read_value(reader)? {
        None | Some(Value::Float(None | Some(Float::Missing))) => Ok(None),
        Some(Value::Float(Some(Float::Value(n)))) => {
            Ok(Some(vcf::record::info::field::Value::Float(n)))
        }
        Some(Value::FloatArray(values)) => Ok(Some(vcf::record::info::field::Value::FloatArray(
            values
                .into_iter()
                .map(Float::from)
                .map(|value| match value {
                    Float::Value(n) => Some(n),
                    Float::Missing => None,
                    _ => todo!("unhandled float array value: {:?}", value),
                })
                .collect(),
        ))),
        v => Err(type_mismatch_error(v, Type::Float)),
    }
}

fn read_info_field_character_value<R>(
    reader: &mut R,
) -> io::Result<Option<vcf::record::info::field::Value>>
where
    R: Read,
{
    const DELIMITER: char = ',';
    const MISSING_VALUE: char = '.';

    match read_value(reader)? {
        None | Some(Value::String(None)) => Ok(None),
        Some(Value::String(Some(s))) => match s.len() {
            0 | 1 => s
                .chars()
                .next()
                .map(vcf::record::info::field::Value::Character)
                .map(|v| Ok(Some(v)))
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "INFO character value missing")
                })?,
            _ => Ok(Some(vcf::record::info::field::Value::CharacterArray(
                s.split(DELIMITER)
                    .flat_map(|t| t.chars())
                    .map(|c| match c {
                        MISSING_VALUE => None,
                        _ => Some(c),
                    })
                    .collect(),
            ))),
        },
        v => Err(type_mismatch_error(v, Type::Character)),
    }
}

fn read_info_field_string_value<R>(
    reader: &mut R,
) -> io::Result<Option<vcf::record::info::field::Value>>
where
    R: Read,
{
    match read_value(reader)? {
        None | Some(Value::String(None)) => Ok(None),
        Some(Value::String(Some(s))) => Ok(Some(vcf::record::info::field::Value::String(s))),
        v => Err(type_mismatch_error(v, Type::String)),
    }
}

fn type_mismatch_error(actual: Option<Value>, expected: Type) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("type mismatch: expected {}, got {:?}", expected, actual),
    )
}

#[cfg(test)]
mod tests {
    use vcf::header::{info::Key, Number};

    use super::*;

    #[test]
    fn test_read_info_field_value_with_integer_value() -> io::Result<()> {
        fn t(
            mut reader: &[u8],
            info: &vcf::header::Info,
            expected_value: Option<i32>,
        ) -> io::Result<()> {
            let actual = read_info_field_value(&mut reader, info)?;
            let expected = expected_value.map(vcf::record::info::field::Value::Integer);
            assert_eq!(actual, expected);
            Ok(())
        }

        let info = vcf::header::Info::from(Key::Other(
            String::from("I32"),
            Number::Count(1),
            Type::Integer,
            String::default(),
        ));

        // None
        t(&[0x00], &info, None)?;

        // Some(Value::Int8(None))
        t(&[0x01], &info, None)?;
        // Some(Value::Int8(Int8::Missing))
        t(&[0x11, 0x80], &info, None)?;
        // Some(Value::Int8(Some(Int8::Value(8))))
        t(&[0x11, 0x08], &info, Some(8))?;

        // Some(Value::Int16(None))
        t(&[0x02], &info, None)?;
        // Some(Value::Int16(Int16::Missing))
        t(&[0x12, 0x00, 0x80], &info, None)?;
        // Some(Value::Int16(Some(Int16::Value(13))))
        t(&[0x12, 0x0d, 0x00], &info, Some(13))?;

        // Some(Value::Int32(None))
        t(&[0x03], &info, None)?;
        // Some(Value::Int32(Int32::Missing))
        t(&[0x13, 0x00, 0x00, 0x00, 0x80], &info, None)?;
        // Some(Value::Int32(Some(Int32::Value(21))))
        t(&[0x13, 0x15, 0x00, 0x00, 0x00], &info, Some(21))?;

        Ok(())
    }

    #[test]
    fn test_read_info_field_value_with_integer_array_value() -> io::Result<()> {
        fn t(
            mut reader: &[u8],
            info: &vcf::header::Info,
            expected_value: Option<Vec<Option<i32>>>,
        ) -> io::Result<()> {
            let actual = read_info_field_value(&mut reader, info)?;
            let expected = expected_value.map(vcf::record::info::field::Value::IntegerArray);
            assert_eq!(actual, expected);
            Ok(())
        }

        let info = vcf::header::Info::from(Key::Other(
            String::from("I32"),
            Number::Count(2),
            Type::Integer,
            String::default(),
        ));

        // Some(Value::IntegerArray([Some(8), Some(13)]))
        t(&[0x21, 0x08, 0x0d], &info, Some(vec![Some(8), Some(13)]))?;
        // Some(Value::IntegerArray([Some(8), None]))
        t(&[0x21, 0x08, 0x80], &info, Some(vec![Some(8), None]))?;

        // Some(Value::IntegerArray([Some(21), Some(34)]))
        t(
            &[0x22, 0x15, 0x00, 0x22, 0x00],
            &info,
            Some(vec![Some(21), Some(34)]),
        )?;
        // Some(Value::IntegerArray([Some(21), None]))
        t(
            &[0x22, 0x15, 0x00, 0x00, 0x80],
            &info,
            Some(vec![Some(21), None]),
        )?;

        // Some(Value::IntegerArray([Some(55), Some(89)]))
        t(
            &[0x23, 0x37, 0x00, 0x00, 0x00, 0x59, 0x00, 0x00, 0x00],
            &info,
            Some(vec![Some(55), Some(89)]),
        )?;
        // Some(Value::IntegerArray([Some(55), None]))
        t(
            &[0x23, 0x37, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80],
            &info,
            Some(vec![Some(55), None]),
        )?;

        Ok(())
    }

    #[test]
    fn test_read_info_field_value_with_flag_value() -> io::Result<()> {
        fn t(mut reader: &[u8], info: &vcf::header::Info) -> io::Result<()> {
            let actual = read_info_field_value(&mut reader, info)?;
            let expected = Some(vcf::record::info::field::Value::Flag);
            assert_eq!(actual, expected);
            Ok(())
        }

        let info = vcf::header::Info::from(Key::Other(
            String::from("BOOL"),
            Number::Count(1),
            Type::Flag,
            String::default(),
        ));

        // None
        t(&[0x00], &info)?;
        // Some(Value::Int8(Some(Int8::Value(1))))
        t(&[0x11, 0x01], &info)?;

        Ok(())
    }

    #[test]
    fn test_read_info_field_value_with_float_value() -> io::Result<()> {
        fn t(
            mut reader: &[u8],
            info: &vcf::header::Info,
            expected_value: Option<f32>,
        ) -> io::Result<()> {
            let actual = read_info_field_value(&mut reader, info)?;
            let expected = expected_value.map(vcf::record::info::field::Value::Float);
            assert_eq!(actual, expected);
            Ok(())
        }

        let info = vcf::header::Info::from(Key::Other(
            String::from("F32"),
            Number::Count(1),
            Type::Float,
            String::default(),
        ));

        // None
        t(&[0x00], &info, None)?;
        // Some(Value::Float(None))
        t(&[0x05], &info, None)?;
        // Some(Value::Float(Some(Float::Missing)))
        t(&[0x15, 0x01, 0x00, 0x80, 0x7f], &info, None)?;

        // Some(Value::Float(Some(Float::Value(0.0))))
        t(&[0x15, 0x00, 0x00, 0x00, 0x00], &info, Some(0.0))?;

        Ok(())
    }

    #[test]
    fn test_read_info_field_value_with_float_array_value() -> io::Result<()> {
        fn t(
            mut reader: &[u8],
            info: &vcf::header::Info,
            expected_value: Option<Vec<Option<f32>>>,
        ) -> io::Result<()> {
            let actual = read_info_field_value(&mut reader, info)?;
            let expected = expected_value.map(vcf::record::info::field::Value::FloatArray);
            assert_eq!(actual, expected);
            Ok(())
        }

        let info = vcf::header::Info::from(Key::Other(
            String::from("F32"),
            Number::Count(2),
            Type::Float,
            String::default(),
        ));

        // Some(Value::FloatArray([0.0, 1.0]))
        t(
            &[0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3f],
            &info,
            Some(vec![Some(0.0), Some(1.0)]),
        )?;
        // Some(Value::FloatArray([0.0, None]))
        t(
            &[0x25, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x80, 0x7f],
            &info,
            Some(vec![Some(0.0), None]),
        )?;

        Ok(())
    }

    #[test]
    fn test_read_info_field_value_with_character_value() -> io::Result<()> {
        fn t(
            mut reader: &[u8],
            info: &vcf::header::Info,
            expected_value: Option<char>,
        ) -> io::Result<()> {
            let actual = read_info_field_value(&mut reader, info)?;
            let expected = expected_value.map(vcf::record::info::field::Value::Character);
            assert_eq!(actual, expected);
            Ok(())
        }

        let info = vcf::header::Info::from(Key::Other(
            String::from("CHAR"),
            Number::Count(1),
            Type::Character,
            String::default(),
        ));

        // None
        t(&[0x00], &info, None)?;
        // Some(Value::String(None))
        t(&[0x07], &info, None)?;

        // Some(Value::String(Some(String::from("n"))))
        t(&[0x17, 0x6e], &info, Some('n'))?;

        Ok(())
    }

    #[test]
    fn test_read_info_field_value_with_character_array_value() -> io::Result<()> {
        fn t(
            mut reader: &[u8],
            info: &vcf::header::Info,
            expected_value: Option<Vec<Option<char>>>,
        ) -> io::Result<()> {
            let actual = read_info_field_value(&mut reader, info)?;
            let expected = expected_value.map(vcf::record::info::field::Value::CharacterArray);
            assert_eq!(actual, expected);
            Ok(())
        }

        let info = vcf::header::Info::from(Key::Other(
            String::from("CHAR"),
            Number::Count(2),
            Type::Character,
            String::default(),
        ));

        // None
        t(&[0x00], &info, None)?;

        // Some(Value::String(Some(String::from("n,d"))))
        t(
            &[0x37, 0x6e, 0x2c, 0x64],
            &info,
            Some(vec![Some('n'), Some('d')]),
        )?;
        // Some(Value::String(Some(String::from("n,."))))
        t(
            &[0x37, 0x6e, 0x2c, 0x2e],
            &info,
            Some(vec![Some('n'), None]),
        )?;

        Ok(())
    }

    #[test]
    fn test_read_info_field_value_with_string_value() -> io::Result<()> {
        fn t(
            mut reader: &[u8],
            info: &vcf::header::Info,
            expected_value: Option<&str>,
        ) -> io::Result<()> {
            let actual = read_info_field_value(&mut reader, info)?;
            let expected =
                expected_value.map(|s| vcf::record::info::field::Value::String(s.into()));
            assert_eq!(actual, expected);
            Ok(())
        }

        let info = vcf::header::Info::from(Key::Other(
            String::from("STRING"),
            Number::Count(1),
            Type::String,
            String::default(),
        ));

        // None
        t(&[0x00], &info, None)?;

        // Some(Value::String(None))
        t(&[0x07], &info, None)?;
        // Some(Value::String(Some(String::from("ndls"))))
        t(&[0x47, 0x6e, 0x64, 0x6c, 0x73], &info, Some("ndls"))?;

        Ok(())
    }
}
