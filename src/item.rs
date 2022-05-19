use anyhow::{Result, anyhow};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::mem;

use crate::{ErrorCode, GetItem};
use crate::read_ext::ReadExt;
use crate::tags::TagGroup;

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
    pub data: Option<Box<dyn Any>>
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
            data: Some(Box::new(data))
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
            DataType::Bitfield => todo!(),
            DataType::String => {
                let mut buf = vec![0u8; data_len as usize];
                reader.read_exact(&mut buf)?;
                Some(Box::new(String::from_utf8(buf)?))
            },
            DataType::Container => {
                let mut items: Vec<Item> = Vec::new();
                let mut container_size = data_len;
                while container_size > 0 {
                    items.push(Item::read_bytes(reader, &mut container_size)?);
                }
                Some(Box::new(items))
            },
            DataType::Timestamp => Some(Box::new(read_timestamp(reader))),
            DataType::ByteArray => {
                let mut buf = vec![0u8; data_len as usize];
                reader.read_exact(&mut buf)?;
                Some(Box::new(buf))
            },
            DataType::Error => Some(Box::new(ErrorCode::from(reader.read_le::<u32>()?))),
        };

        *length -= data_len + ITEM_HEADER_SIZE;

        Ok(Self {
            tag: tag & TAG_MASK,
            data: data
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
                _ => Box::new("None")
            }
        },
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
        DataType::Bitfield => todo!(),
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
fn get_data_type (data: Option<&Box<dyn Any>>) -> Result<DataType> {
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
                x if x == TypeId::of::<str>() => Err(anyhow!("Invalid data type use String instead of str")),
                x if x == TypeId::of::<String>() => Ok(DataType::String),
                x if x == TypeId::of::<Vec<Item>>() => Ok(DataType::Container),
                x if x == TypeId::of::<DateTime<Utc>>() => Ok(DataType::Timestamp),
                x if x == TypeId::of::<[u8]>() => Err(anyhow!("Invalid data type use Vec<u8> instead of [u8]")),
                x if x == TypeId::of::<Vec<u8>>() => Ok(DataType::ByteArray),
                x if x == TypeId::of::<ErrorCode>() => Ok(DataType::Error),
                _ => Err(anyhow!("Invalid data type"))
            }
        },
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
            DataType::None => (),
            DataType::Bool => {
                let b = p.downcast_ref::<bool>().unwrap();
                if *b {
                    writer.write(&[0x01u8])?;
                } else {
                    writer.write(&[0x00u8])?;
                }
            },
            DataType::Char8 => {
                writer.write(&p.downcast_ref::<i8>().unwrap().to_le_bytes())?;
            },
            DataType::UChar8 => {
                writer.write(&p.downcast_ref::<u8>().unwrap().to_le_bytes())?;
            },
            DataType::Int16 => {
                writer.write(&p.downcast_ref::<i16>().unwrap().to_le_bytes())?;
            },
            DataType::UInt16 => {
                writer.write(&p.downcast_ref::<u16>().unwrap().to_le_bytes())?;
            },
            DataType::Int32 => {
                writer.write(&p.downcast_ref::<i32>().unwrap().to_le_bytes())?;
            },
            DataType::UInt32 => {
                writer.write(&p.downcast_ref::<u32>().unwrap().to_le_bytes())?;
            },
            DataType::Int64 => {
                writer.write(&p.downcast_ref::<i64>().unwrap().to_le_bytes())?;
            },
            DataType::UInt64 => {
                writer.write(&p.downcast_ref::<u64>().unwrap().to_le_bytes())?;
            },
            DataType::Float32 => {
                writer.write(&p.downcast_ref::<f32>().unwrap().to_le_bytes())?;
            },
            DataType::Double64 => {
                writer.write(&p.downcast_ref::<f64>().unwrap().to_le_bytes())?;
            },
            DataType::Bitfield => todo!(),
            DataType::String => {
                writer.write(&p.downcast_ref::<String>().unwrap().as_bytes())?;
            },
            DataType::Container => {
                let items = data.unwrap().downcast_ref::<Vec<Item>>().unwrap();
                for item in items {
                    item.write_bytes(writer)?;
                }
            },
            DataType::Timestamp =>  {
                write_timestamp(writer, p.downcast_ref::<DateTime<Utc>>().unwrap())?;
            },
            DataType::ByteArray => {
                writer.write(&p.downcast_ref::<Vec<u8>>().unwrap())?;
            },
            DataType::Error => {
                writer.write(&(*p.downcast_ref::<ErrorCode>().unwrap() as u32).to_le_bytes())?;
            },
        }
    }

    Ok(())
}

/// Writes datetime to writer
/// 
/// # Arguments
/// 
/// `writer` - write cursor
/// `date_time` - the time to write
pub fn write_timestamp<W: Write>(writer: &mut W, date_time: &DateTime<Utc>) -> Result<()> {
    writer.write(&date_time.timestamp().to_le_bytes())?;
    writer.write(&date_time.timestamp_subsec_nanos().to_le_bytes())?;
    Ok(())
}

/// Reads datetime from reader
/// 
/// # Arguments
/// 
/// `reader` - the reader
pub fn read_timestamp<R: Read>(reader: &mut R) -> Result<DateTime<Utc>> {
    let seconds = reader.read_le::<i64>()?;
    let nanos = reader.read_le::<u32>()?;
    Ok(DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(seconds, nanos), Utc))
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
macro_rules! test_data_cases {
    () => {{
        let test_cases: Vec<(DataType, Option<Box<dyn Any>>, Vec<u8>, u16)> = vec![
            (DataType::None, None, vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], mem::size_of::<()>() as u16),
            (DataType::Bool, Some(Box::new(true)), vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x01], mem::size_of::<bool>() as u16),
            (DataType::Bool, Some(Box::new(false)), vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00], mem::size_of::<bool>() as u16),
            (DataType::Char8, Some(Box::new(-1i8)), vec![0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x00, 0xff], mem::size_of::<i8>() as u16),
            (DataType::UChar8, Some(Box::new(1u8)), vec![0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x00, 0x01], mem::size_of::<u8>() as u16),
            (DataType::Int16, Some(Box::new(-1i16)), vec![0x00, 0x00, 0x00, 0x00, 0x04, 0x02, 0x00, 0xff, 0xff], mem::size_of::<i16>() as u16),
            (DataType::UInt16, Some(Box::new(1u16)), vec![0x00, 0x00, 0x00, 0x00, 0x05, 0x02, 0x00, 0x01, 0x00], mem::size_of::<u16>() as u16),
            (DataType::Int32, Some(Box::new(-1i32)), vec![0x00, 0x00, 0x00, 0x00, 0x06, 0x04, 0x00, 0xff, 0xff, 0xff, 0xff], mem::size_of::<i32>() as u16),
            (DataType::UInt32, Some(Box::new(1u32)), vec![0x00, 0x00, 0x00, 0x00, 0x07, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00], mem::size_of::<u32>() as u16),
            (DataType::Int64, Some(Box::new(-1i64)), vec![0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], mem::size_of::<i64>() as u16),
            (DataType::UInt64, Some(Box::new(1u64)), vec![0x00, 0x00, 0x00, 0x00, 0x09, 0x08, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], mem::size_of::<u64>() as u16),
            (DataType::Float32, Some(Box::new(1.0f32)), vec![0x00, 0x00, 0x00, 0x00, 0x0a, 0x04, 0x00, 0x00, 0x00, 0x80, 0x3f], mem::size_of::<f32>() as u16),
            (DataType::Double64, Some(Box::new(1.0f64)), vec![0x00, 0x00, 0x00, 0x00, 0x0b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f], mem::size_of::<f64>() as u16),
            // bitfield, not implemented
            (DataType::String, Some(Box::new("Test".to_string())), vec![0x00, 0x00, 0x00, 0x00, 0x0d, 0x04, 0x00, 0x54, 0x65, 0x73, 0x74], 4),
            (DataType::Container, Some(Box::new(vec![
                Item::new(crate::tags::RSCP::AUTHENTICATION_USER.into(), "user".to_string()),
                Item::new(crate::tags::RSCP::AUTHENTICATION_PASSWORD.into(), "pwd".to_string()),
            ])), vec![0x00, 0x00, 0x00, 0x00, 0x0e, 0x15, 0x00, 2, 0, 0, 0, 13, 4, 0, 117, 115, 101, 114, 3, 0, 0, 0, 13, 3, 0, 112, 119, 100], 21),
            (DataType::Timestamp, Some(Box::new(DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(12345678, 123456), Utc))), vec![0x00, 0x00, 0x00, 0x00, 0x0f, 0x0c, 0x00, 78, 97, 188, 0, 0, 0, 0, 0, 64, 226, 1, 0], (mem::size_of::<i64>() + mem::size_of::<i32>()) as u16),
            (DataType::ByteArray, Some(Box::new(vec![0x0fu8; 4])), vec![0x00, 0x00, 0x00, 0x00, 0x10, 0x04, 0x00, 0x0f, 0x0f, 0x0f, 0x0f], 4),
            (DataType::Error, Some(Box::new(ErrorCode::NotHandled)), vec![0x00, 0x00, 0x00, 0x00, 0xff, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00], mem::size_of::<u32>() as u16),
        ];
        test_cases
    }};
}

#[test]
fn test_item_from_bytes() {
    let test_cases = test_data_cases!();
    for (data_type, _, data_buffer, _) in test_cases {
        let mut buffer_size = data_buffer.len() as u16;
        let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(data_buffer);
        let item = Item::read_bytes(&mut buffer, &mut buffer_size).unwrap();
        assert_eq!(item.tag, 0x00, "Test tag {:?}", data_type);
        // TODO: test data against source
    }
}

#[test]
fn test_item_to_bytes() {
    let test_cases = test_data_cases!();
    for (data_type, data, result, _) in test_cases {
        let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        let item = Item { tag: 0x00, data: data };
        item.write_bytes(&mut buffer).unwrap();
        assert_eq!(buffer.get_ref().to_vec(), result, "Test {:?}", data_type);
    };
}

#[test]
fn test_get_data_length() {
    let test_cases = test_data_cases!();
    for (data_type, data, _, data_size) in test_cases {
        let data_size_from_data = get_data_length(&data_type, data.as_ref()).unwrap();
        assert_eq!(data_size, data_size_from_data, "Test {:?}", data_type);
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
    for (data_type, data, _, _) in test_cases {
        let data_type_from_data = get_data_type(data.as_ref()).unwrap();
        assert_eq!(data_type, data_type_from_data, "Test {:?}", data_type);
    }
}

#[test]
fn test_write_data() {    
    let test_cases = test_data_cases!();
    for (data_type, data, result, _) in test_cases {
        let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        write_data(&mut buffer, &data_type, data.as_ref()).unwrap();
        assert_eq!(buffer.get_ref().to_vec(), result[ITEM_HEADER_SIZE as usize..], "Test {:?}", data_type);
    }
}

#[test]
fn test_write_timestamp() {    
    let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    write_timestamp(&mut buffer, &DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(12345678, 123456), Utc)).unwrap();
    assert_eq!(buffer.get_ref().to_vec(), vec![78, 97, 188, 0, 0, 0, 0, 0, 64, 226, 1, 0]);
}

#[test]
fn test_read_timestamp() {    
    let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(vec![78, 97, 188, 0, 0, 0, 0, 0, 64, 226, 1, 0]);
    let date_time = read_timestamp(&mut buffer).unwrap();
    assert_eq!(date_time.timestamp(), 12345678);
    assert_eq!(date_time.timestamp_subsec_nanos(), 123456);
}

