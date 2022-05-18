
use anyhow::{Result, bail};
use log::{info, warn, debug};
use std::io::{Write, Read};
use std::net::{TcpStream, ToSocketAddrs, Shutdown};
use std::sync::{Mutex, Arc};

use crate::encryption::{RscpEncryption, BLOCK_SIZE};
use crate::{tags, Item, Frame, Errors, UserLevel};
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

        info!("Authenticate");
        info!("{:?}", frame);
        match self.send_receive_frame(&frame) {
            Ok(result_frame) => { 
                info!("{:?}", result_frame);
                let user_level = result_frame.get_item_data::<u8>(tags::RSCP::AUTHENTICATION.into()).unwrap();
                let user_level_type = UserLevel::from(user_level.clone());
                info!("Authenticated as {:?}", user_level_type);
            },
            Err(_) => {
                self.disconnect()?;
                bail!(Errors::AuthFailed)
            }
        }        
        
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
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
                Err(e) => { break; } //return Err(anyhow!("error receiving data: {}", e))
            }
        };
        Ok(data)
    }

    fn send_receive_frame(&mut self, frame: &Frame) -> Result<Frame> {

        let data = frame.to_bytes()?;
        debug!("<< Frame: {:02x?}", data);
        let enc_data = self.enc_processor.encrypt(data)?;

        self.write_to_stream(&enc_data)?;
        let return_enc_data = self.read_from_stream()?;
        if return_enc_data.len() == 0 {
            bail!(Errors::ReceiveNothing)
        }

        let return_data = self.enc_processor.decrypt(return_enc_data)?;
        debug!(">> Frame: {:02x?}", return_data);

        Ok(Frame::from_bytes(return_data)?)
    }
}
