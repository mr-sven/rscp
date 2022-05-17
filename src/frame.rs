
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use crc::{Crc, CRC_32_ISO_HDLC};
use std::any::Any;
use std::io::Cursor;
use std::io::Write;

use crate::Item;
use crate::item::{DataType, get_data_length, read_timestamp, write_timestamp, write_data};
use crate::read_ext::ReadExt;

const MAGIC_ID: u16 = 0xE3DC;
const PROTOCOL_VERSION: u8 = 0x01;
const PROTOCOL_VERSION_MASK: u8 = 0x0F;
const WITH_CHECKSUM: u8 = 0x10;

pub struct Frame {
    pub with_checksum: bool,
    pub time_stamp: DateTime<Utc>,
    pub items: Option<Box<dyn Any>> 
}

impl Frame {
    pub fn new() -> Self {
        Self {
            with_checksum: true,
            time_stamp: Utc::now(),
            items: Some(Box::new(Vec::new() as Vec<Item>))
        }
    }

    pub fn push_item(&mut self, item: Item) {
        let items_box = self.items.as_mut().unwrap();
        let items_vector = items_box.downcast_mut::<Vec<Item>>().unwrap();
        items_vector.push(item);
    }

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

        // calculates CRC sum
        let sum = crc_sum.checksum(buffer.get_ref());
        
        // write crc sum
        buffer.write(&sum.to_le_bytes())?;

        Ok(buffer.get_ref().to_vec())
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {

        let mut buffer: Cursor<Vec<u8>> = Cursor::new(data);
        let crc_sum: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

        // magic ID is big endian
        if buffer.read_be::<u16>()? != MAGIC_ID {
            return Err(anyhow!("Invalid magic header"));
        }

        // documentation missmatch of version flag
        buffer.read_le::<u8>()?;

        // protocol version and checksum flag
        let prot_ver = buffer.read_le::<u8>()?;
        if prot_ver & PROTOCOL_VERSION_MASK != PROTOCOL_VERSION {
            return Err(anyhow!("Invalid Protocol version, got {:?}", prot_ver));
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
            return Err(anyhow!("CRC Checksum missmatch, got {:?} = {:?}", cksum, sum));
        }

        // set position back to data
        buffer.set_position(data_start);

        // parse items
        let mut items: Vec<Item> = Vec::new();
        let mut container_size = length;
        while container_size > 0 {
            items.push(Item::from_bytes(&mut buffer, &mut container_size)?);
        }

        Ok(Self {
            with_checksum: with_checksum,
            time_stamp: time_stamp,
            items: Some(Box::new(items))
        })
    }
}

