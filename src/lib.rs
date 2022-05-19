#[macro_use] extern crate macro_attr;

use anyhow::{anyhow, Result};
use std::any::Any;
use std::fmt::Debug;

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

macro_rules! user_level_ext {
    (
        $(#[$($attrs:tt)*])*
        pub enum $name:ident { $($vn:ident = $v:tt),+ }
    ) => {
        /// Error code result
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

user_level_ext! {
    #[derive(Copy, Clone)]
    #[derive(PartialEq, Debug)]
    #[repr(u8)]
    pub enum UserLevel {
        NotAuthorized = 0,
        User = 10,
        Installer = 20,
        Service = 30,
        Admin = 40,
        E3dc = 50,
        E3dcRoot = 60,
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

#[derive(Debug)] // Allow the use of "{:?}" format specifier
pub enum Errors {
    Parse(String),
    ReceiveNothing,
    AuthFailed
}

impl std::error::Error for Errors {}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Errors::Parse(ref msg) => write!(f, "Frame parse error: {}", msg),
            Errors::ReceiveNothing => write!(f, "Receive nothing"),
            Errors::AuthFailed => write!(f, "Authentication failed")
        }
    }
}

/// ################################################
///      TEST TEST TEST
/// ################################################

#[test]
fn test_error_code() {
    assert_eq!(ErrorCode::from(0x01u32), ErrorCode::NotHandled, "Test From<u32>");
    assert_eq!(Into::<u32>::into(ErrorCode::NotHandled), 0x01u32, "Test Into<u32>");
    assert_eq!(ErrorCode::from(0xffffu32), ErrorCode::Unknown, "Test From Unknown<u32>");
}

#[test]
fn test_user_level() {
    assert_eq!(UserLevel::from(10), UserLevel::User, "Test From<u8>");
    assert_eq!(Into::<u8>::into(UserLevel::User), 10, "Test Into<u8>");
    assert_eq!(UserLevel::from(0xfe), UserLevel::Unknown, "Test From Unknown<u8>");
}