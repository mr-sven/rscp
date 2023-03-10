use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use crc::{Crc, CRC_32_ISO_HDLC};
use std::any::Any;
use std::fmt::Debug;
use std::io::Cursor;
use std::io::Write;

use crate::item::{get_data_length, read_timestamp, write_data, write_timestamp, DataType};
use crate::read_ext::ReadExt;
use crate::{Errors, GetItem, Item};

/// the protocol magic id for rscp frame
const MAGIC_ID: u16 = 0xE3DC;

/// version of protocol
const PROTOCOL_VERSION: u8 = 0x01;

/// bitmask for protocol
const PROTOCOL_VERSION_MASK: u8 = 0x0F;

/// with checksum flag of frame
const WITH_CHECKSUM: u8 = 0x10;

/// RSCP data frame
pub struct Frame {
    /// true if frame contains checksum
    pub with_checksum: bool,

    /// the timestamp of the frame
    pub time_stamp: DateTime<Utc>,

    /// contains data items
    pub items: Option<Box<dyn Any>>,
}

impl Frame {
    /// Returns a frame
    ///
    /// # Examples
    ///
    /// ```
    /// use rscp::Frame;
    /// let frame = Frame::new();
    /// ```
    pub fn new() -> Self {
        Self {
            with_checksum: true,
            time_stamp: Utc::now(),
            items: Some(Box::new(Vec::new() as Vec<Item>)),
        }
    }

    /// Appends data item to current frame
    ///
    /// # Arguments
    ///
    /// * `item` - the data item
    ///
    /// # Examples
    ///
    /// ```
    /// use rscp::{tags, Item, Frame};
    /// let mut info_frame = Frame::new();
    /// info_frame.push_item(Item { tag: tags::INFO::SERIAL_NUMBER.into(), data: None } );
    /// ```
    pub fn push_item(&mut self, item: Item) {
        let items_box = self.items.as_mut().unwrap();
        let items_vector = items_box.downcast_mut::<Vec<Item>>().unwrap();
        items_vector.push(item);
    }

    /// Returns data frame a byte vector
    ///
    /// # Examples
    ///
    /// ```
    /// use rscp::{tags, Item, Frame};
    /// let mut info_frame = Frame::new();
    /// info_frame.push_item(Item { tag: tags::INFO::SERIAL_NUMBER.into(), data: None } );
    /// let frame_bytes = info_frame.to_bytes();
    /// ```
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let data_length = get_data_length(&DataType::Container, self.items.as_ref())?;
        let crc_sum: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        // magic ID is big endian
        buffer.write(&MAGIC_ID.to_be_bytes())?;

        // documentation missmatch of version flag
        buffer.write(&[0x00u8])?;

        // add protocol version and checksum flag
        if self.with_checksum {
            buffer.write(&[PROTOCOL_VERSION | WITH_CHECKSUM])?;
        } else {
            buffer.write(&[PROTOCOL_VERSION])?;
        }

        // write timestamp to data
        write_timestamp(&mut buffer, &self.time_stamp)?;

        // writes the current
        buffer.write(&data_length.to_le_bytes())?;

        // writes the container data
        write_data(&mut buffer, &DataType::Container, self.items.as_ref())?;

        if self.with_checksum {
            // calculates CRC sum
            let sum = crc_sum.checksum(buffer.get_ref());

            // write crc sum
            buffer.write(&sum.to_le_bytes())?;
        }

        Ok(buffer.get_ref().to_vec())
    }

    /// Returns data frame from a byte vector
    ///
    /// # Examples
    ///
    /// ```
    /// use rscp::Frame;
    /// let frame = Frame::from_bytes(vec![0xe3, 0xdc, 0x00, 0x11, 0x95, 0x23, 0x86, 0x62, 0x00, 0x00, 0x00, 0x00, 0x90, 0x1d, 0x45, 0x35, 0x08, 0x00, 0x01, 0x00, 0x80, 0x00, 0x03, 0x01, 0x00, 0x0a, 0x0f, 0x24, 0x01, 0x23, 0x00, 0x00]);
    /// ```
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let mut buffer: Cursor<Vec<u8>> = Cursor::new(data);
        let crc_sum: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

        // magic ID is big endian
        if buffer.read_be::<u16>()? != MAGIC_ID {
            bail!(Errors::Parse("Invalid magic header".to_string()))
        }

        // documentation missmatch of version flag
        buffer.read_le::<u8>()?;

        // protocol version and checksum flag
        let prot_ver = buffer.read_le::<u8>()?;
        if prot_ver & PROTOCOL_VERSION_MASK != PROTOCOL_VERSION {
            bail!(Errors::Parse(format!(
                "Invalid Protocol version, got {:?}",
                prot_ver
            )))
        }

        let with_checksum = if prot_ver & WITH_CHECKSUM == WITH_CHECKSUM {
            true
        } else {
            false
        };

        // read timestamp
        let time_stamp = read_timestamp(&mut buffer)?;

        // read data length
        let length = buffer.read_le::<u16>()?;

        if with_checksum {
            // save current data length
            let data_start = buffer.position();

            // length of data for checksum calc
            let data_check_length = length as usize + data_start as usize;

            // set position to start
            buffer.set_position(0);

            // calculate checksum
            let sum = crc_sum.checksum(&buffer.get_ref()[..data_check_length]);

            // move position to checksum
            buffer.set_position(data_check_length as u64);

            // read checksum
            let cksum = buffer.read_le::<u32>()?;
            if cksum != sum {
                bail!(Errors::Parse(format!(
                    "CRC Checksum missmatch, got {:?} = {:?}",
                    cksum, sum
                )))
            }

            // set position back to data
            buffer.set_position(data_start);
        }

        // parse items
        let mut items: Vec<Item> = Vec::new();
        let mut container_size = length;
        while container_size > 0 {
            items.push(Item::read_bytes(&mut buffer, &mut container_size)?);
        }

        Ok(Self {
            with_checksum: with_checksum,
            time_stamp: time_stamp,
            items: Some(Box::new(items)),
        })
    }
}

impl Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = self.get_data::<Vec<Item>>().unwrap();
        f.debug_struct("Frame")
            .field("time_stamp", &self.time_stamp)
            .field("items", items)
            .finish()
    }
}

// implementation for frame object, accesses data object functions
impl GetItem for Frame {
    fn get_data<T: 'static + Sized>(&self) -> Result<&T> {
        Ok(self.items.get_data()?)
    }

    fn get_item(&self, tag: u32) -> Result<&Item> {
        Ok(self.items.get_item(tag)?)
    }

    fn get_item_data<T: 'static + Sized>(&self, tag: u32) -> Result<&T> {
        Ok(self.items.get_item_data(tag)?)
    }
}

/// ################################################
///      TEST TEST TEST
/// ################################################

#[test]
fn test_new() {
    let frame = Frame::new();
    assert_eq!(frame.with_checksum, true);
    assert_eq!(
        frame
            .items
            .unwrap()
            .downcast_ref::<Vec<Item>>()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn test_push_item() {
    let mut frame = Frame::new();
    frame.push_item(Item {
        tag: crate::tags::INFO::SERIAL_NUMBER.into(),
        data: None,
    });
    assert_eq!(
        frame
            .items
            .unwrap()
            .downcast_ref::<Vec<Item>>()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn test_to_bytes() {
    let frame = Frame {
        with_checksum: true,
        time_stamp: DateTime::<Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(12345678, 123456),
            Utc,
        ),
        items: Some(Box::new(vec![Item {
            tag: crate::tags::INFO::SERIAL_NUMBER.into(),
            data: None,
        }])),
    };
    assert_eq!(
        frame.to_bytes().unwrap(),
        vec![
            0xe3, 0xdc, 0x00, 0x11, 0x4e, 0x61, 0xbc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0xe2,
            0x01, 0x00, 0x07, 0x00, 0x01, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0xfe, 0xfa, 0x84,
            0x3c
        ]
    );

    let frame = Frame {
        with_checksum: false,
        time_stamp: DateTime::<Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(12345678, 123456),
            Utc,
        ),
        items: Some(Box::new(vec![Item {
            tag: crate::tags::INFO::SERIAL_NUMBER.into(),
            data: None,
        }])),
    };
    assert_eq!(
        frame.to_bytes().unwrap(),
        vec![
            0xe3, 0xdc, 0x00, 0x01, 0x4e, 0x61, 0xbc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0xe2,
            0x01, 0x00, 0x07, 0x00, 0x01, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00
        ]
    );
}

#[test]
fn test_from_bytes() {
    let frame = Frame::from_bytes(vec![
        0xe3, 0xdc, 0x00, 0x11, 0x4e, 0x61, 0xbc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0xe2, 0x01,
        0x00, 0x07, 0x00, 0x01, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0xfe, 0xfa, 0x84, 0x3c,
    ])
    .unwrap();
    assert_eq!(frame.with_checksum, true);
    assert_eq!(
        frame
            .items
            .unwrap()
            .downcast_ref::<Vec<Item>>()
            .unwrap()
            .len(),
        1
    );

    let frame = Frame::from_bytes(vec![
        0xe3, 0xdc, 0x00, 0x01, 0x4e, 0x61, 0xbc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0xe2, 0x01,
        0x00, 0x07, 0x00, 0x01, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00,
    ])
    .unwrap();
    assert_eq!(frame.with_checksum, false);

    let frame_err = Frame::from_bytes(vec![0xaa, 0xdc, 0x00, 0x00]);
    assert_eq!(
        format!("{}", frame_err.unwrap_err().downcast::<Errors>().unwrap()),
        "Frame parse error: Invalid magic header"
    );

    let frame_err = Frame::from_bytes(vec![0xe3, 0xdc, 0x00, 0x00]);
    assert_eq!(
        format!("{}", frame_err.unwrap_err().downcast::<Errors>().unwrap()),
        "Frame parse error: Invalid Protocol version, got 0"
    );

    let frame_err = Frame::from_bytes(vec![
        0xe3, 0xdc, 0x00, 0x11, 0x4e, 0x61, 0xbc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0xe2, 0x01,
        0x00, 0x07, 0x00, 0x01, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0xfe, 0xfa, 0x84, 0x33,
    ]);
    assert_eq!(
        format!("{}", frame_err.unwrap_err().downcast::<Errors>().unwrap()),
        "Frame parse error: CRC Checksum missmatch, got 864353022 = 1015347966"
    );
}

#[test]
fn test_debug_impl() {
    let frame = Frame {
        with_checksum: true,
        time_stamp: DateTime::<Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(12345678, 123456),
            Utc,
        ),
        items: Some(Box::new(vec![Item {
            tag: crate::tags::INFO::SERIAL_NUMBER.into(),
            data: None,
        }])),
    };
    assert_eq!(format!("{:?}", frame), "Frame { time_stamp: 1970-05-23T21:21:18.000123456Z, items: [Item { tag: \"INFO_SERIAL_NUMBER\", data: \"None\" }] }");
}

#[test]
fn test_get_item_impl() {
    let frame = Frame {
        with_checksum: true,
        time_stamp: DateTime::<Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(12345678, 123456),
            Utc,
        ),
        items: Some(Box::new(vec![Item::new(
            crate::tags::INFO::SERIAL_NUMBER.into(),
            "serial".to_string(),
        )])),
    };

    let item = frame
        .get_item(crate::tags::INFO::SERIAL_NUMBER.into())
        .unwrap();
    assert_eq!(item.get_data::<String>().unwrap(), "serial");
    assert_eq!(
        frame
            .get_item_data::<String>(crate::tags::INFO::SERIAL_NUMBER.into())
            .unwrap(),
        "serial"
    );
}
