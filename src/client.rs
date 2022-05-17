
use anyhow::{anyhow, Result};
use log::{info, warn};
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Mutex, Arc};

use crate::encryption::{RscpEncryption, BLOCK_SIZE};
use crate::{tags, Item, Frame};
use crate::GetItem;

const DEFAULT_PORT: u16 = 5033;

pub struct Client<'a> {
    pub connected: bool,
    enc_processor: RscpEncryption,
    connection: Option<Arc<Mutex<TcpStream>>>,
    username: &'a str,
    password: &'a str,
}

impl<'a> Client<'a> {
    pub fn new(rscp_key: &str, username: &'a str, pasword: &'a str) -> Self {
        Self {
            connected: false,
            connection: None,
            enc_processor: RscpEncryption::new(rscp_key),
            username: username,
            password: pasword
        }
    }

    pub fn connect(&mut self, host: &str, port: Option<u16>) -> Result<()> {
        let host_port = port.unwrap_or(DEFAULT_PORT);
        let addr_list = format!("{}:{}", host, host_port).to_socket_addrs()?;
        let addr = addr_list.last().unwrap();
        info!("Connect to {}:{}", host, host_port);

        let stream = TcpStream::connect(addr)?;
        self.connected = true;
        self.connection = Some(Arc::new(Mutex::new(stream)));
        info!("Connected");
        
        let mut frame = Frame::new();
        frame.push_item(Item::new(tags::RSCP::AUTHENTICATION.into(), vec![
            Item::new(tags::RSCP::AUTHENTICATION_USER.into(), self.username.to_string()),
            Item::new(tags::RSCP::AUTHENTICATION_PASSWORD.into(), self.password.to_string()),
        ]));

        let data = self.enc_processor.encrypt(frame.to_bytes()?)?;

        info!("Authenticate");
        self.write_to_stream(&data)?;

        let mut data = [0 as u8; BLOCK_SIZE];

        Ok(())
    }

    fn write_to_stream(&mut self, data: &[u8]) -> Result<()> {        
        self.connection.as_mut().unwrap()
            .as_ref().lock().unwrap().by_ref()
            .write(&data)?;
        Ok(())
    }
}
