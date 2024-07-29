use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::mem;

use crate::read_ext::ReadExt;
use crate::tags::TagGroup;
use crate::{ErrorCode, GetItem};

/// Site of item header - tag: 4, type: 1, length; 2
const ITEM_HEADER_SIZE: u16 = 7;

/// bitmask to drop response bit
const TAG_MASK: u32 = 0xff7fffff;

macro_rules! data_type_ext {
    (
        $(#[$($attrs:tt)*])*
        pub enum $name:ident { $($vn:ident = $v:tt),+ }
    ) => {
        $(#[$($attrs)*])*
        pub enum $name {
            $($vn = $v),+
        }

        impl Into<u8> for $name {
            fn into(self) -> u8 {
                self as u8
            }
        }

        impl From<u8> for $name {
            fn from(orig: u8) -> Self {
                match orig {
                    $(x if x == $name::$vn as u8 => $name::$vn,)*
                    _ => $name::Error
                }
            }
        }
    }
}

data_type_ext! {
    #[derive(Copy, Clone, Debug, PartialEq)]
    #[repr(u8)]
    pub enum DataType {
        None = 0x00,
        Bool = 0x01,
        Char8 = 0x02,
        UChar8 = 0x03,
        Int16 = 0x04,
        UInt16 = 0x05,
        Int32 = 0x06,
        UInt32 = 0x07,
        Int64 = 0x08,
        UInt64 = 0x09,
        Float32 = 0x0A,
        Double64 = 0x0B,
        Bitfield = 0x0C,
        String = 0x0D,
        Container = 0x0E,
        Timestamp = 0x0F,
        ByteArray = 0x10,
        Error = 0xFF
    }
}

/// RSCP data item
pub struct Item {
    /// Tag identifier
    pub tag: u32,

    /// data content
    pub data: Option<Box<dyn Any>>,
}

impl Item {
    /// Returns a data item using tag and any data element
    ///
    /// # Arguments
    ///
    /// * `tag` - u32 representation of RSCP Protocol Tag
    /// * `data` - Any data content
    ///
    /// # Examples
    ///
    /// ```
    /// use rscp::{tags, Item};
    /// let item = Item::new(tags::RSCP::AUTHENTICATION_USER.into(), "username".to_string());
    /// // item with none content
    /// let item_none = Item { tag: tags::INFO::SERIAL_NUMBER.into(), data: None };
    /// ```
    pub fn new<T: Any>(tag: u32, data: T) -> Self {
        Self {
            tag: tag,
            data: Some(Box::new(data)),
        }
    }

    /// Writes data to write cursor
    ///
    /// # Arguments
    ///
    /// * `writer` - write cursor
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    /// let item = Item::new(tags::RSCP::AUTHENTICATION_USER.into(), "username".to_string());
    /// item.write_bytes(&mut buffer)?;
    /// ```
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<()> {
        // write tag to buffer
        writer.write(&self.tag.to_le_bytes())?;

        // get the current type of data and write to buffer
        let data_type = get_data_type(self.data.as_ref())?;
        writer.write(&[data_type.into()])?;

        // get the lenght of the data and write to buffer
        let data_length = get_data_length(&data_type, self.data.as_ref())?;
        writer.write(&data_length.to_le_bytes())?;

        // write the data to buffer
        write_data(writer, &data_type, self.data.as_ref())?;

        Ok(())
    }

    /// Returns a data item from read cursor
    ///
    /// # Arguments
    ///
    /// * `reader` - read cursor
    /// * `length` - pointer to current size of remaining data, will be decremented by number of bytes processed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use rscp::Item;
    /// let mut buffer: Cursor<Vec<u8>> = Cursor::new(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    /// let mut len: u16 = 7;
    /// let item = Item::read_bytes(&mut buffer, &mut len);
    /// ```
    pub fn read_bytes<R: Read>(reader: &mut R, length: &mut u16) -> Result<Self> {
        let tag = reader.read_le::<u32>()?;
        let data_type = DataType::from(reader.read_le::<u8>()?);
        let data_len = reader.read_le::<u16>()?;

        let data: Option<Box<dyn Any>> = match data_type {
            DataType::None => None,
            DataType::Bool => Some(Box::new(reader.read_le::<u8>()? == 0x01)),
            DataType::Char8 => Some(Box::new(reader.read_le::<i8>()?)),
            DataType::UChar8 => Some(Box::new(reader.read_le::<u8>()?)),
            DataType::Int16 => Some(Box::new(reader.read_le::<i16>()?)),
            DataType::UInt16 => Some(Box::new(reader.read_le::<u16>()?)),
            DataType::Int32 => Some(Box::new(reader.read_le::<i32>()?)),
            DataType::UInt32 => Some(Box::new(reader.read_le::<u32>()?)),
            DataType::Int64 => Some(Box::new(reader.read_le::<i64>()?)),
            DataType::UInt64 => Some(Box::new(reader.read_le::<u64>()?)),
            DataType::Float32 => Some(Box::new(reader.read_le::<f32>()?)),
            DataType::Double64 => Some(Box::new(reader.read_le::<f64>()?)),
            DataType::Bitfield => Some(Box::new(read_bitfield(reader, data_len)?)),
            DataType::String => {
                let mut buf = vec![0u8; data_len as usize];
                reader.read_exact(&mut buf)?;
                Some(Box::new(String::from_utf8(buf)?))
            }
            DataType::Container => {
                let mut items: Vec<Item> = Vec::new();
                let mut container_size = data_len;
                while container_size > 0 {
                    items.push(Item::read_bytes(reader, &mut container_size)?);
                }
                Some(Box::new(items))
            }
            DataType::Timestamp => Some(Box::new(read_timestamp(reader)?)),
            DataType::ByteArray => {
                let mut buf = vec![0u8; data_len as usize];
                reader.read_exact(&mut buf)?;
                Some(Box::new(buf))
            }
            DataType::Error => Some(Box::new(ErrorCode::from(reader.read_le::<u32>()?))),
        };

        *length -= data_len + ITEM_HEADER_SIZE;

        Ok(Self {
            tag: tag & TAG_MASK,
            data: data,
        })
    }
}

/// implementation for item object, accesses data object functions
impl GetItem for Item {
    fn get_data<T: 'static + Sized>(&self) -> Result<&T> {
        Ok(self.data.get_data()?)
    }

    fn get_item(&self, tag: u32) -> Result<&Item> {
        Ok(self.data.get_item(tag)?)
    }

    fn get_item_data<T: 'static + Sized>(&self, tag: u32) -> Result<&T> {
        Ok(self.data.get_item_data(tag)?)
    }
}

impl std::fmt::Debug for Item {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let tag_group = TagGroup::from((&self.tag >> 24) as u8);
        let data_debug = get_debug_data(self.data.as_ref());

        fmt.debug_struct("Item")
            .field("tag", &tag_group.tags(&self.tag & TAG_MASK))
            .field("data", &data_debug)
            .finish()
    }
}

impl Clone for Item {
    fn clone(&self) -> Self {
        let data_type = get_data_type(self.data.as_ref()).unwrap();
        let clone = match data_type {
            DataType::Bool => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<bool>().unwrap().clone())) },
            DataType::Char8 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<i8>().unwrap().clone())) },
            DataType::UChar8 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<u8>().unwrap().clone())) },
            DataType::Int16 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<i16>().unwrap().clone())) },
            DataType::UInt16 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<u16>().unwrap().clone())) },
            DataType::Int32 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<i32>().unwrap().clone())) },
            DataType::UInt32 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<u32>().unwrap().clone())) },
            DataType::Int64 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<i64>().unwrap().clone())) },
            DataType::UInt64 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<u64>().unwrap().clone())) },
            DataType::Float32 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<f32>().unwrap().clone())) },
            DataType::Double64 => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<f64>().unwrap().clone())) },
            DataType::Bitfield => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<Vec<bool>>().unwrap().clone())) },
            DataType::String => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<String>().unwrap().clone())) },
            DataType::Container => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<Vec<Item>>().unwrap().clone())) },
            DataType::Timestamp => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<DateTime<Utc>>().unwrap().clone())) },
            DataType::ByteArray => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<Vec<u8>>().unwrap().clone())) },
            DataType::Error => Self { tag: self.tag, data: Some(Box::new(self.data.get_data::<ErrorCode>().unwrap().clone())) },
            DataType::None => Self { tag: self.tag, data: None },
        };
        return clone;
    }
}

/// helper function for std::fmt::Debug of Item
///
/// # Arguments
///
/// * `data` - Any data type to convert
fn get_debug_data(data: Option<&Box<dyn Any>>) -> Box<dyn Debug + '_> {
    match data {
        Some(p) => {
            let current_id = (&**p).type_id();
            match current_id {
                x if x == TypeId::of::<bool>() => Box::new(p.downcast_ref::<bool>().unwrap()),
                x if x == TypeId::of::<i8>() => Box::new(p.downcast_ref::<i8>().unwrap()),
                x if x == TypeId::of::<u8>() => Box::new(p.downcast_ref::<u8>().unwrap()),
                x if x == TypeId::of::<i16>() => Box::new(p.downcast_ref::<i16>().unwrap()),
                x if x == TypeId::of::<u16>() => Box::new(p.downcast_ref::<u16>().unwrap()),
                x if x == TypeId::of::<i32>() => Box::new(p.downcast_ref::<i32>().unwrap()),
                x if x == TypeId::of::<u32>() => Box::new(p.downcast_ref::<u32>().unwrap()),
                x if x == TypeId::of::<i64>() => Box::new(p.downcast_ref::<i64>().unwrap()),
                x if x == TypeId::of::<u64>() => Box::new(p.downcast_ref::<u64>().unwrap()),
                x if x == TypeId::of::<f32>() => Box::new(p.downcast_ref::<f32>().unwrap()),
                x if x == TypeId::of::<f64>() => Box::new(p.downcast_ref::<f64>().unwrap()),
                x if x == TypeId::of::<Vec<bool>>() => Box::new(p.downcast_ref::<Vec<bool>>().unwrap()),
                x if x == TypeId::of::<String>() => Box::new(p.downcast_ref::<String>().unwrap()),
                x if x == TypeId::of::<Vec<Item>>() => Box::new(p.downcast_ref::<Vec<Item>>().unwrap()),
                x if x == TypeId::of::<DateTime<Utc>>() => Box::new(p.downcast_ref::<DateTime<Utc>>().unwrap()),
                x if x == TypeId::of::<Vec<u8>>() => Box::new(p.downcast_ref::<Vec<u8>>().unwrap()),
                x if x == TypeId::of::<ErrorCode>() => Box::new(p.downcast_ref::<ErrorCode>().unwrap()),
                _ => Box::new("None"),
            }
        }
        None => Box::new("None"),
    }
}

/// retuns the lenght of the data by DataType and size at string, container and byte array
///
/// # Arguments
///
/// * `data_type` - type of data
/// * `data` - Any data, required for string, container and byte array
pub fn get_data_length(data_type: &DataType, data: Option<&Box<dyn Any>>) -> Result<u16> {
    match data_type {
        DataType::None => Ok(mem::size_of::<()>() as u16),
        DataType::Bool => Ok(mem::size_of::<bool>() as u16),
        DataType::Char8 => Ok(mem::size_of::<i8>() as u16),
        DataType::UChar8 => Ok(mem::size_of::<u8>() as u16),
        DataType::Int16 => Ok(mem::size_of::<i16>() as u16),
        DataType::UInt16 => Ok(mem::size_of::<u16>() as u16),
        DataType::Int32 => Ok(mem::size_of::<i32>() as u16),
        DataType::UInt32 => Ok(mem::size_of::<u32>() as u16),
        DataType::Int64 => Ok(mem::size_of::<i64>() as u16),
        DataType::UInt64 => Ok(mem::size_of::<u64>() as u16),
        DataType::Float32 => Ok(mem::size_of::<f32>() as u16),
        DataType::Double64 => Ok(mem::size_of::<f64>() as u16),
        DataType::Bitfield => Ok(((data.unwrap().downcast_ref::<Vec<bool>>().unwrap().len() as f32) / 8.0).ceil() as u16),
        DataType::String => Ok(data.unwrap().downcast_ref::<String>().unwrap().len() as u16),
        DataType::Container => Ok(get_container_size(data.unwrap().downcast_ref::<Vec<Item>>().unwrap())?),
        DataType::Timestamp => Ok((mem::size_of::<i64>() + mem::size_of::<i32>()) as u16),
        DataType::ByteArray => Ok(data.unwrap().downcast_ref::<Vec<u8>>().unwrap().len() as u16),
        DataType::Error => Ok(mem::size_of::<u32>() as u16),
    }
}

/// retuns the size of a item vector (Container)
///
/// # Arguments
///
/// * `items` - Vector of items
fn get_container_size(items: &[Item]) -> Result<u16> {
    let mut size: u16 = 0;
    for item in items {
        size += ITEM_HEADER_SIZE;
        let data_type = get_data_type(item.data.as_ref())?;
        size += get_data_length(&data_type, item.data.as_ref())?;
    }
    Ok(size)
}

/// retuns data type of Any
///
/// # Arguments
///
/// * `data` - Any Option
fn get_data_type(data: Option<&Box<dyn Any>>) -> Result<DataType> {
    match data {
        Some(p) => {
            // double deref: first * removes ref to Box, second * removes ref from box to any
            let actual_id = (&**p).type_id();
            match actual_id {
                x if x == TypeId::of::<bool>() => Ok(DataType::Bool),
                x if x == TypeId::of::<i8>() => Ok(DataType::Char8),
                x if x == TypeId::of::<u8>() => Ok(DataType::UChar8),
                x if x == TypeId::of::<i16>() => Ok(DataType::Int16),
                x if x == TypeId::of::<u16>() => Ok(DataType::UInt16),
                x if x == TypeId::of::<i32>() => Ok(DataType::Int32),
                x if x == TypeId::of::<u32>() => Ok(DataType::UInt32),
                x if x == TypeId::of::<i64>() => Ok(DataType::Int64),
                x if x == TypeId::of::<u64>() => Ok(DataType::UInt64),
                x if x == TypeId::of::<f32>() => Ok(DataType::Float32),
                x if x == TypeId::of::<f64>() => Ok(DataType::Double64),
                x if x == TypeId::of::<Vec<bool>>() => Ok(DataType::Bitfield),
                x if x == TypeId::of::<String>() => Ok(DataType::String),
                x if x == TypeId::of::<Vec<Item>>() => Ok(DataType::Container),
                x if x == TypeId::of::<DateTime<Utc>>() => Ok(DataType::Timestamp),
                x if x == TypeId::of::<Vec<u8>>() => Ok(DataType::ByteArray),
                x if x == TypeId::of::<ErrorCode>() => Ok(DataType::Error),
                _ => Err(anyhow!("Invalid data type")),
            }
        }
        None => Ok(DataType::None),
    }
}

/// Write data to write cursor
///
/// # Arguments
///
/// * `writer` - Write cursor
/// * `data_type` - type of data
/// * `data` - the data to write
pub fn write_data<W: Write>(writer: &mut W, data_type: &DataType, data: Option<&Box<dyn Any>>) -> Result<()> {
    if let Some(p) = data {
        match data_type {
            DataType::None => {},
            DataType::Bool => { if *p.downcast_ref::<bool>().unwrap() {
                writer.write(&[0x01u8])?;
            } else {
                writer.write(&[0x00u8])?;
            }},
            DataType::Char8 => { writer.write(&p.downcast_ref::<i8>().unwrap().to_le_bytes())?; },
            DataType::UChar8 => { writer.write(&p.downcast_ref::<u8>().unwrap().to_le_bytes())?; },
            DataType::Int16 => { writer.write(&p.downcast_ref::<i16>().unwrap().to_le_bytes())?; },
            DataType::UInt16 => { writer.write(&p.downcast_ref::<u16>().unwrap().to_le_bytes())?; },
            DataType::Int32 => { writer.write(&p.downcast_ref::<i32>().unwrap().to_le_bytes())?; },
            DataType::UInt32 => { writer.write(&p.downcast_ref::<u32>().unwrap().to_le_bytes())?; },
            DataType::Int64 => { writer.write(&p.downcast_ref::<i64>().unwrap().to_le_bytes())?; },
            DataType::UInt64 => { writer.write(&p.downcast_ref::<u64>().unwrap().to_le_bytes())?; },
            DataType::Float32 => { writer.write(&p.downcast_ref::<f32>().unwrap().to_le_bytes())?; },
            DataType::Double64 => { writer.write(&p.downcast_ref::<f64>().unwrap().to_le_bytes())?; },
            DataType::Bitfield => { write_bitfield(writer, p.downcast_ref::<Vec<bool>>().unwrap())?; },
            DataType::String => { writer.write(&p.downcast_ref::<String>().unwrap().as_bytes())?; },
            DataType::Container => { for item in data.unwrap().downcast_ref::<Vec<Item>>().unwrap() {
                item.write_bytes(writer)?;
            }},
            DataType::Timestamp => { write_timestamp(writer, p.downcast_ref::<DateTime<Utc>>().unwrap())?; },
            DataType::ByteArray => { writer.write(&p.downcast_ref::<Vec<u8>>().unwrap())?; },
            DataType::Error => { writer.write(&(*p.downcast_ref::<ErrorCode>().unwrap() as u32).to_le_bytes())?; },
        }
    }

    Ok(())
}

/// Writes bitfield to writer
///
/// # Arguments
///
/// * `writer` - write cursor
/// * `bits` - vector of bits
fn write_bitfield<W: Write>(writer: &mut W, bits: &[bool]) -> Result<()> {
    let mut bytes = vec![0u8; (bits.len() as f32 / 8.0).ceil() as usize];
    for bit_index in 0..bits.len() {
        if bits[bit_index] {
            let byte_index = bit_index / 8;
            bytes[byte_index] |= 1 << (bit_index % 8);
        }
    }
    writer.write(&bytes)?;
    Ok(())
}

/// Reads bitfield
///
/// # Arguments
///
/// * `reader` - the reader
/// * `data_len` - length of data
fn read_bitfield<R: Read>(reader: &mut R, data_len: u16) -> Result<Vec<bool>> {
    let mut buf = vec![0u8; data_len as usize];
    reader.read_exact(&mut buf)?;

    let mut bits = vec![false; data_len as usize * 8];

    for byte_index in 0..data_len {
        for bit_index in 0..8 {
            bits[(byte_index * 8 + bit_index) as usize] = buf[byte_index as usize] & (1 << bit_index) != 0;
        }
    }
    Ok(bits)
}

/// Writes datetime to writer
///
/// # Arguments
///
/// * `writer` - write cursor
/// * `date_time` - the time to write
pub fn write_timestamp<W: Write>(writer: &mut W, date_time: &DateTime<Utc>) -> Result<()> {
    writer.write(&date_time.timestamp().to_le_bytes())?;
    writer.write(&date_time.timestamp_subsec_nanos().to_le_bytes())?;
    Ok(())
}

/// Reads datetime from reader
///
/// # Arguments
///
/// * `reader` - the reader
pub fn read_timestamp<R: Read>(reader: &mut R) -> Result<DateTime<Utc>> {
    let seconds = reader.read_le::<i64>()?;
    let nanos = reader.read_le::<u32>()?;
    Ok(DateTime::<Utc>::from_timestamp(seconds, nanos).unwrap())
}

/// ################################################
///      TEST TEST TEST
/// ################################################

#[test]
fn test_data_type() {
    assert_eq!(DataType::from(0x01), DataType::Bool, "Test From<u8>");
    assert_eq!(Into::<u8>::into(DataType::Bool), 0x01, "Test Into<u8>");
    assert_eq!(DataType::from(0xfe), DataType::Error, "Test From<u8>");
}

#[cfg(test)]
struct TestData {
    data_type: DataType,
    data: Option<Box<dyn Any>>,
    byte_data: Vec<u8>,
    data_size: u16,
    item_str: &'static str,
}

#[cfg(test)]
macro_rules! test_data_cases {
    () => {{
        let test_cases = vec![
            TestData {
                data_type: DataType::None,
                data: None,
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                data_size: mem::size_of::<()>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: \"None\" }",
            },
            TestData {
                data_type: DataType::Bool,
                data: Some(Box::new(true)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x01],
                data_size: mem::size_of::<bool>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: true }",
            },
            TestData {
                data_type: DataType::Bool,
                data: Some(Box::new(false)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00],
                data_size: mem::size_of::<bool>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: false }",
            },
            TestData {
                data_type: DataType::Char8,
                data: Some(Box::new(-1i8)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x00, 0xff],
                data_size: mem::size_of::<i8>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: -1 }",
            },
            TestData {
                data_type: DataType::UChar8,
                data: Some(Box::new(1u8)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x00, 0x01],
                data_size: mem::size_of::<u8>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: 1 }",
            },
            TestData {
                data_type: DataType::Int16,
                data: Some(Box::new(-1i16)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x04, 0x02, 0x00, 0xff, 0xff],
                data_size: mem::size_of::<i16>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: -1 }",
            },
            TestData {
                data_type: DataType::UInt16,
                data: Some(Box::new(1u16)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x05, 0x02, 0x00, 0x01, 0x00],
                data_size: mem::size_of::<u16>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: 1 }",
            },
            TestData {
                data_type: DataType::Int32,
                data: Some(Box::new(-1i32)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x06, 0x04, 0x00, 0xff, 0xff, 0xff, 0xff],
                data_size: mem::size_of::<i32>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: -1 }",
            },
            TestData {
                data_type: DataType::UInt32,
                data: Some(Box::new(1u32)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x07, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00],
                data_size: mem::size_of::<u32>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: 1 }",
            },
            TestData {
                data_type: DataType::Int64,
                data: Some(Box::new(-1i64)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
                data_size: mem::size_of::<i64>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: -1 }",
            },
            TestData {
                data_type: DataType::UInt64,
                data: Some(Box::new(1u64)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x09, 0x08, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                data_size: mem::size_of::<u64>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: 1 }",
            },
            TestData {
                data_type: DataType::Float32,
                data: Some(Box::new(1.0f32)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x0a, 0x04, 0x00, 0x00, 0x00, 0x80, 0x3f],
                data_size: mem::size_of::<f32>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: 1.0 }",
            },
            TestData {
                data_type: DataType::Double64,
                data: Some(Box::new(1.0f64)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x0b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f],
                data_size: mem::size_of::<f64>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: 1.0 }",
            },
            TestData {
                data_type: DataType::Bitfield,
                data: Some(Box::new(vec![true, false, true, false, true, false, true, false, false, true, false, true, false, true, false, true])),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x0c, 0x02, 0x00, 0x55, 0xaa],
                data_size: 2,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: [true, false, true, false, true, false, true, false, false, true, false, true, false, true, false, true] }",
            },
            TestData {
                data_type: DataType::String,
                data: Some(Box::new("Test".to_string())),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x0d, 0x04, 0x00, 0x54, 0x65, 0x73, 0x74],
                data_size: 4,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: \"Test\" }",
            },
            TestData {
                data_type: DataType::Container,
                data: Some(Box::new(vec![
                    Item::new(crate::tags::RSCP::AUTHENTICATION_USER.into(), "user".to_string()),
                    Item::new(crate::tags::RSCP::AUTHENTICATION_PASSWORD.into(), "pwd".to_string()),
                ])),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x0e, 0x15, 0x00, 2, 0, 0, 0, 13, 4, 0, 117, 115, 101, 114, 3, 0, 0, 0, 13, 3, 0, 112, 119, 100],
                data_size: 21,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: [Item { tag: \"RSCP_AUTHENTICATION_USER\", data: \"user\" }, Item { tag: \"RSCP_AUTHENTICATION_PASSWORD\", data: \"pwd\" }] }",
            },
            TestData {
                data_type: DataType::Timestamp,
                data: Some(Box::new(DateTime::<Utc>::from_timestamp(12345678, 123456).unwrap())),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x0f, 0x0c, 0x00, 78, 97, 188, 0, 0, 0, 0, 0, 64, 226, 1, 0],
                data_size: (mem::size_of::<i64>() + mem::size_of::<i32>()) as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: 1970-05-23T21:21:18.000123456Z }",
            },
            TestData {
                data_type: DataType::ByteArray,
                data: Some(Box::new(vec![0x0fu8; 4])),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0x10, 0x04, 0x00, 0x0f, 0x0f, 0x0f, 0x0f],
                data_size: 4,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: [15, 15, 15, 15] }",
            },
            TestData {
                data_type: DataType::Error,
                data: Some(Box::new(ErrorCode::NotHandled)),
                byte_data: vec![0x00, 0x00, 0x00, 0x00, 0xff, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00],
                data_size: mem::size_of::<u32>() as u16,
                item_str: "Item { tag: \"RSCP_GENERAL_ERROR\", data: NotHandled }",
            },
        ];
        test_cases
    }};
}

#[test]
fn test_item_write_bytes() {
    let test_cases = test_data_cases!();
    for test_case in test_cases {
        let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        let item = Item { tag: 0x00, data: test_case.data };
        item.write_bytes(&mut buffer).unwrap();
        assert_eq!(buffer.get_ref().to_vec(), test_case.byte_data, "Test {:?}", test_case.data_type);
    }
}

#[test]
fn test_item_read_bytes() {
    let test_cases = test_data_cases!();
    for test_case in test_cases {
        let mut buffer_size = test_case.byte_data.len() as u16;
        let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(test_case.byte_data);
        let item = Item::read_bytes(&mut buffer, &mut buffer_size).unwrap();
        assert_eq!(item.tag, 0x00, "Test tag {:?}", test_case.data_type);
        // TODO: test data against source
    }
}

#[test]
fn test_get_item_impl() {
    let item_container = Item::new(crate::tags::RSCP::AUTHENTICATION.into(), vec![
        Item::new(crate::tags::RSCP::AUTHENTICATION_USER.into(), "username".to_string()),
        Item::new(crate::tags::RSCP::AUTHENTICATION_PASSWORD.into(), "password".to_string()),
    ]);

    let sub_item = item_container.get_item(crate::tags::RSCP::AUTHENTICATION_USER.into()).unwrap();
    assert_eq!(sub_item.get_data::<String>().unwrap(), "username");
    assert_eq!(item_container.get_item_data::<String>(crate::tags::RSCP::AUTHENTICATION_USER.into()).unwrap(), "username");

    let err_item = item_container.get_item(crate::tags::RSCP::GENERAL_ERROR.into());
    assert_eq!(err_item.unwrap_err().downcast::<String>().unwrap(), "Tag not found 8388607");
}

#[test]
fn test_display_impl() {
    let test_cases = test_data_cases!();
    for test_case in test_cases {
        let item = Item { tag: 0x00, data: test_case.data };
        assert_eq!(format!("{:?}", item), test_case.item_str);
    }

    let item = Item { tag: 0x00, data: Some(Box::new([1u8, 2, 3, 4, 5])) };
    assert_eq!(format!("{:?}", item), "Item { tag: \"RSCP_GENERAL_ERROR\", data: \"None\" }");
}

#[test]
fn test_get_data_length() {
    let test_cases = test_data_cases!();
    for test_case in test_cases {
        let data_size_from_data = get_data_length(&test_case.data_type, test_case.data.as_ref()).unwrap();
        assert_eq!(test_case.data_size, data_size_from_data, "Test {:?}", test_case.data_type);
    }
}

#[test]
fn test_get_container_size() {
    let container_size = get_container_size(&vec![
        Item::new(crate::tags::RSCP::AUTHENTICATION_USER.into(), "user".to_string()),
        Item::new(crate::tags::RSCP::AUTHENTICATION_PASSWORD.into(), "pwd".to_string()),
    ]).unwrap();
    assert_eq!(container_size, 21);
}

#[test]
fn test_get_data_type() {
    let test_cases = test_data_cases!();
    for test_case in test_cases {
        let data_type_from_data = get_data_type(test_case.data.as_ref()).unwrap();
        assert_eq!(test_case.data_type, data_type_from_data, "Test {:?}", test_case.data_type);
    }

    let fail_vec_box: Box<dyn Any> = Box::new([1u8, 2, 3, 4, 5]);
    let fail_vec = get_data_type(Some(&fail_vec_box));
    assert_eq!(fail_vec.unwrap_err().downcast::<&str>().unwrap(), "Invalid data type");
}

#[test]
fn test_write_data() {
    let test_cases = test_data_cases!();
    for test_case in test_cases {
        let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        write_data(&mut buffer, &test_case.data_type, test_case.data.as_ref()).unwrap();
        assert_eq!(buffer.get_ref().to_vec(), test_case.byte_data[ITEM_HEADER_SIZE as usize..], "Test {:?}", test_case.data_type);
    }

    let none_vec_box: Box<dyn Any> = Box::new(0);
    let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    write_data(&mut buffer, &DataType::None, Some(&none_vec_box)).unwrap();
    assert_eq!(buffer.get_ref().to_vec(), Vec::new(), "Test None {:?}", DataType::None);
}

#[test]
fn test_write_bitfield() {
    let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    write_bitfield(&mut buffer, &vec![true, false, true, false, true, false, true, false, false, true, false, true, false, true, false, true]).unwrap();
    assert_eq!(buffer.get_ref().to_vec(), vec![0b01010101, 0b10101010]);
}

#[test]
fn test_read_bitfield() {
    let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(vec![0b01010101, 0b10101010]);
    let bits = read_bitfield(&mut buffer, 2).unwrap();
    assert_eq!(bits[0], true);
    assert_eq!(bits[1], false);
    assert_eq!(bits[2], true);
    assert_eq!(bits[3], false);
    assert_eq!(bits[4], true);
    assert_eq!(bits[5], false);
    assert_eq!(bits[6], true);
    assert_eq!(bits[7], false);
    assert_eq!(bits[8], false);
    assert_eq!(bits[9], true);
    assert_eq!(bits[10], false);
    assert_eq!(bits[11], true);
    assert_eq!(bits[12], false);
    assert_eq!(bits[13], true);
    assert_eq!(bits[14], false);
    assert_eq!(bits[15], true);
}

#[test]
fn test_write_timestamp() {
    let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    write_timestamp(&mut buffer, &DateTime::<Utc>::from_timestamp(12345678, 123456).unwrap()).unwrap();
    assert_eq!(buffer.get_ref().to_vec(), vec![78, 97, 188, 0, 0, 0, 0, 0, 64, 226, 1, 0]);
}

#[test]
fn test_read_timestamp() {
    let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(vec![78, 97, 188, 0, 0, 0, 0, 0, 64, 226, 1, 0]);
    let date_time = read_timestamp(&mut buffer).unwrap();
    assert_eq!(date_time.timestamp(), 12345678);
    assert_eq!(date_time.timestamp_subsec_nanos(), 123456);
}
