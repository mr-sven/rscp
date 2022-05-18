
use anyhow::{anyhow, Result};
use log::{info, warn};
use std::io::{Write, Read};
use std::net::{TcpStream, ToSocketAddrs, Shutdown};
use std::sync::{Mutex, Arc};

use crate::encryption::{RscpEncryption, BLOCK_SIZE};
use crate::{tags, Item, Frame};
use crate::GetItem;

const DEFAULT_PORT: u16 = 5033;

pub struct Client {
    pub connected: bool,
    enc_processor: RscpEncryption,
    connection: Option<Arc<Mutex<TcpStream>>>,
    username: String,
    password: String,
}

impl Client {
    pub fn new(rscp_key: &str, username: String, pasword: String) -> Self {
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
        stream.set_read_timeout(Some(std::time::Duration::from_millis(500)))?;
        self.connected = true;
        self.connection = Some(Arc::new(Mutex::new(stream)));
        info!("Connected");
        
        let mut frame = Frame::new();
        frame.push_item(Item::new(tags::RSCP::AUTHENTICATION.into(), vec![
            Item::new(tags::RSCP::AUTHENTICATION_USER.into(), self.username.to_string()),
            Item::new(tags::RSCP::AUTHENTICATION_PASSWORD.into(), self.password.to_string()),
        ]));

        let mut data = self.enc_processor.encrypt(frame.to_bytes()?)?;

        info!("Authenticate");
        self.write_to_stream(&data)?;
        data = self.read_from_stream()?;

        println!("{:02x?}", data);
        let dev = self.enc_processor.decrypt(data)?;
        println!("{:02x?}", dev);
        let frm = Frame::from_bytes(dev)?;
        
        self.connection.as_mut().unwrap().as_ref().lock().unwrap().shutdown(Shutdown::Both)?;
        
        Ok(())
    }

    fn write_to_stream(&mut self, data: &[u8]) -> Result<()> { 
        self.connection.as_mut().unwrap().as_ref().lock().unwrap().write(&data)?;
        Ok(())
    }

    fn read_from_stream(&mut self) -> Result<Vec<u8>> {
        let mut buffer = [0 as u8; BLOCK_SIZE];
        let mut data: Vec<u8> = Vec::new();
        loop
        {
            match self.connection.as_mut().unwrap().as_ref().lock().unwrap().read_exact(&mut buffer) {
                Ok(_) => { data.extend_from_slice(&buffer); },
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => { break; },
                Err(e) => {break;}//return Err(anyhow!("error receiving data: {}", e))
            }
        };
        Ok(data)
    }
}
