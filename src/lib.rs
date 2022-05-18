#[macro_use] extern crate macro_attr;

use anyhow::{anyhow, Result};
use std::any::Any;

pub mod tags;

mod read_ext;
mod encryption;
mod client;
mod item;
mod frame;

pub use client::Client;
pub use item::Item;
pub use frame::Frame;

macro_rules! error_code_ext {
    (
        $(#[$($attrs:tt)*])*
        pub enum $name:ident { $($vn:ident = $v:tt),+ }
    ) => {
        /// Error code result
        $(#[$($attrs)*])*
        pub enum $name {
            $($vn = $v),+
        }

        impl Into<u32> for $name {
            fn into(self) -> u32 {
                self as u32
            }
        }

        impl From<u32> for $name {
            fn from(orig: u32) -> Self {
                match orig {
                    $(x if x == $name::$vn as u32 => $name::$vn,)*
                    _ => $name::Unknown
                }
            }
        }
    }
}

error_code_ext! {
    #[derive(Copy, Clone)]
    #[derive(PartialEq, Debug)]
    #[repr(u32)]
    pub enum ErrorCode {
        NotHandled = 0x01,
        AccessDenied = 0x02,
        Format = 0x03,
        Again = 0x04,
        OutOfBounds = 0x05,
        NotAvailable = 0x06,
        UnknownTag = 0x07,
        AlreadyInUse = 0x08,
        Unknown = 0xff
    }
}

/// Item and data getter for Frame and Item
pub trait GetItem {
    /// returns typed data from data property
    fn get_data<T: 'static + Sized>(&self) -> Result<&T>;

    /// returns item by tag from data / item list
    fn get_item(&self, tag: u32) -> Result<&Item>;
    
    /// returns typed item data by tag from data / item list
    fn get_item_data<T: 'static + Sized>(&self, tag: u32) -> Result<&T>;
}

/// implementation for data object
impl GetItem for Option<Box<dyn Any>> {

    fn get_data<T: 'static + Sized>(&self) -> Result<&T> {
        Ok(self.as_ref().unwrap().as_ref().downcast_ref::<T>().unwrap())
    }

    fn get_item(&self, tag: u32) -> Result<&Item> {
        let items = self.as_ref().unwrap().downcast_ref::<Vec<Item>>().unwrap();
        for item in items {
            if item.tag == tag {
                return Ok(item);
            }
        }
        Err(anyhow!("Tag not found {:?}", tag))
    }

    fn get_item_data<T: 'static + Sized>(&self, tag: u32) -> Result<&T> {
        let item = self.get_item(tag)?;
        Ok(item.data.as_ref().unwrap().as_ref().downcast_ref::<T>().unwrap())
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

// implementation for item object, accesses data object functions
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

#[derive(Debug)] // Allow the use of "{:?}" format specifier
pub enum Errors {
    Parse(String),
    ReceiveNothing
}

impl std::error::Error for Errors {}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Errors::Parse(ref msg) => write!(f, "Frame parse error: {}", msg),
            Errors::ReceiveNothing => write!(f, "Receive Nothing")
        }
    }
}

#[test]
fn test_error_code() {
    assert_eq!(ErrorCode::from(0x01u32), ErrorCode::NotHandled, "Test From<u32>");
    assert_eq!(Into::<u32>::into(ErrorCode::NotHandled), 0x01u32, "Test Into<u32>");
}