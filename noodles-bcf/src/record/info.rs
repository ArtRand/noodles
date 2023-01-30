use std::io;

use noodles_vcf as vcf;

use crate::header::string_maps::StringStringMap;

/// BCF record info.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Info {
    buf: Vec<u8>,
    field_count: usize,
}

impl Info {
    /// Converts BCF record info to VCF record info.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bcf as bcf;
    /// use noodles_vcf as vcf;
    ///
    /// let bcf_info = bcf::record::Info::default();
    /// let header = vcf::Header::default();
    /// let string_maps = bcf::header::StringMaps::default();
    ///
    /// let vcf_info = bcf_info.try_into_vcf_record_info(&header, string_maps.strings())?;
    /// assert!(vcf_info.is_empty());
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn try_into_vcf_record_info(
        &self,
        header: &vcf::Header,
        string_string_map: &StringStringMap,
    ) -> io::Result<vcf::record::Info> {
        use crate::reader::record::read_info;
        let mut reader = &self.buf[..];
        read_info(&mut reader, header.infos(), string_string_map, self.len())
    }

    /// Creates an info map by wrapping the given buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf::record::Info;
    ///
    /// let data = vec![
    ///     0x11, 0x01, 0x11, 0x05, // AC=5
    ///     0x11, 0x02, 0x11, 0x08, // DP=8
    /// ];
    ///
    /// let info = Info::new(data, 2);
    /// ```
    pub fn new(buf: Vec<u8>, field_count: usize) -> Self {
        Self { buf, field_count }
    }

    /// Returns the number of info fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf::record::Info;
    /// let info = Info::default();
    /// assert_eq!(info.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.field_count
    }

    /// Returns whether there are any info fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf::record::Info;
    /// let info = Info::default();
    /// assert!(info.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes all fields from the info map.
    ///
    /// This does not affect the capacity of the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf::record::Info;
    /// let mut info = Info::default();
    /// info.clear();
    /// assert!(info.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.buf.clear();
        self.set_field_count(0);
    }

    /// Returns the field with the given key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bcf::{header::StringMaps, record::Info};
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::{info::Key, record::value::{map, Map}},
    ///     record::info::{field::Value, Field},
    /// };
    ///
    /// let header = vcf::Header::builder()
    ///     .add_info(Key::AlleleCount, Map::<map::Info>::from(&Key::AlleleCount))
    ///     .add_info(Key::TotalDepth, Map::<map::Info>::from(&Key::TotalDepth))
    ///     .build();
    ///
    /// let string_maps = StringMaps::from(&header);
    ///
    /// let data = vec![
    ///     0x11, 0x01, 0x11, 0x05, // AC=5
    ///     0x11, 0x02, 0x11, 0x08, // DP=8
    /// ];
    ///
    /// let info = Info::new(data, 2);
    ///
    /// assert_eq!(
    ///     info.get(&header, string_maps.strings(), &Key::AlleleCount).transpose()?,
    ///     Some(Field::new(Key::AlleleCount, Some(Value::Integer(5))))
    /// );
    ///
    /// assert!(info.get(&header, string_maps.strings(), &Key::AncestralAllele).is_none());
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn get(
        &self,
        header: &vcf::Header,
        string_string_map: &StringStringMap,
        key: &vcf::header::info::Key,
    ) -> Option<io::Result<vcf::record::info::Field>> {
        for result in self.iter(header, string_string_map) {
            match result {
                Ok((k, v)) => {
                    if &k == key {
                        let field = vcf::record::info::Field::new(k, v);
                        return Some(Ok(field));
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }

        None
    }

    /// Returns an iterator over all info fields.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bcf::{header::StringMaps, record::Info};
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::{info::Key, record::value::{map, Map}},
    ///     record::info::field::Value,
    /// };
    ///
    /// let header = vcf::Header::builder()
    ///     .add_info(Key::AlleleCount, Map::<map::Info>::from(&Key::AlleleCount))
    ///     .add_info(Key::TotalDepth, Map::<map::Info>::from(&Key::TotalDepth))
    ///     .build();
    ///
    /// let string_maps = StringMaps::from(&header);
    ///
    /// let data = vec![
    ///     0x11, 0x01, 0x11, 0x05, // AC=5
    ///     0x11, 0x02, 0x11, 0x08, // DP=8
    /// ];
    ///
    /// let info = Info::new(data, 2);
    /// let mut fields = info.iter(&header, string_maps.strings());
    ///
    /// assert_eq!(
    ///     fields.next().transpose()?,
    ///     Some((Key::AlleleCount, Some(Value::Integer(5))))
    /// );
    ///
    /// assert_eq!(
    ///     fields.next().transpose()?,
    ///     Some((Key::TotalDepth, Some(Value::Integer(8))))
    /// );
    ///
    /// assert!(fields.next().is_none());
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn iter<'a>(
        &'a self,
        header: &'a vcf::Header,
        string_string_map: &'a StringStringMap,
    ) -> impl Iterator<
        Item = io::Result<(
            vcf::header::info::Key,
            Option<vcf::record::info::field::Value>,
        )>,
    > + 'a {
        use crate::reader::record::info::read_info_field;

        let mut reader = &self.buf[..];

        (0..self.len())
            .map(move |_| read_info_field(&mut reader, header.infos(), string_string_map))
    }

    /// Returns an iterator over all info values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bcf::{header::StringMaps, record::Info};
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::{info::Key, record::value::{map, Map}},
    ///     record::info::field::Value,
    /// };
    ///
    /// let header = vcf::Header::builder()
    ///     .add_info(Key::AlleleCount, Map::<map::Info>::from(&Key::AlleleCount))
    ///     .add_info(Key::TotalDepth, Map::<map::Info>::from(&Key::TotalDepth))
    ///     .build();
    ///
    /// let string_maps = StringMaps::from(&header);
    ///
    /// let data = vec![
    ///     0x11, 0x01, 0x11, 0x05, // AC=5
    ///     0x11, 0x02, 0x11, 0x08, // DP=8
    /// ];
    ///
    /// let info = Info::new(data, 2);
    ///
    /// let mut fields = info.values(&header, string_maps.strings());
    /// assert_eq!(fields.next().transpose()?, Some(Some(Value::Integer(5))));
    /// assert_eq!(fields.next().transpose()?, Some(Some(Value::Integer(8))));
    /// assert!(fields.next().is_none());
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn values<'a>(
        &'a self,
        header: &'a vcf::Header,
        string_string_map: &'a StringStringMap,
    ) -> impl Iterator<Item = io::Result<Option<vcf::record::info::field::Value>>> + 'a {
        self.iter(header, string_string_map)
            .map(|result| result.map(|(_, value)| value))
    }

    pub(crate) fn set_field_count(&mut self, field_count: usize) {
        self.field_count = field_count;
    }
}

impl AsRef<[u8]> for Info {
    fn as_ref(&self) -> &[u8] {
        &self.buf
    }
}

impl AsMut<Vec<u8>> for Info {
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }
}
