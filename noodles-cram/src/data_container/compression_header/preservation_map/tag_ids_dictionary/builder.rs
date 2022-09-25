use std::collections::HashMap;

use super::{Key, TagIdsDictionary};
use crate::Record;

#[derive(Debug, Default)]
pub struct Builder {
    keys_indices: HashMap<Vec<Key>, usize>,
}

impl Builder {
    pub fn update(&mut self, record: &Record) {
        let keys: Vec<_> = record.tags().values().map(|field| field.into()).collect();
        let next_index = self.keys_indices.len();
        self.keys_indices.entry(keys).or_insert(next_index);
    }

    pub(crate) fn build(self) -> TagIdsDictionary {
        let mut lines: Vec<_> = self.keys_indices.into_iter().collect();
        lines.sort_by_key(|(_, index)| *index);
        let dictionary: Vec<_> = lines.into_iter().map(|(keys, _)| keys).collect();
        TagIdsDictionary::from(dictionary)
    }
}

#[cfg(test)]
mod tests {
    use noodles_sam::record::data::{
        field::{value::Type, Tag, Value},
        Field,
    };

    use super::*;

    #[test]
    fn test_from_records() {
        let mut builder = Builder::default();

        let mut record = Record::default();
        record
            .tags
            .insert(Field::new(Tag::AlignmentHitCount, Value::Int8(1)));
        builder.update(&record);

        let mut record = Record::default();
        record
            .tags
            .insert(Field::new(Tag::AlignmentHitCount, Value::Int8(2)));
        builder.update(&record);

        let mut record = Record::default();
        record
            .tags
            .insert(Field::new(Tag::AlignmentHitCount, Value::Int8(1)));
        record.tags.insert(Field::new(
            Tag::Comment,
            Value::String(String::from("noodles")),
        ));
        builder.update(&record);

        let dictionary = builder.build();

        assert_eq!(
            *dictionary,
            [
                vec![Key::new(Tag::AlignmentHitCount, Type::Int8)],
                vec![
                    Key::new(Tag::AlignmentHitCount, Type::Int8),
                    Key::new(Tag::Comment, Type::String)
                ]
            ]
        )
    }
}
