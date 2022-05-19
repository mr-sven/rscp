
use anyhow::{Result, bail};
use log::{info, debug};
use std::io::{Write, Read};
use std::net::{TcpStream, ToSocketAddrs, Shutdown};
use std::sync::{Mutex, Arc};

use crate::encryption::{RscpEncryption, BLOCK_SIZE};
use crate::{tags, Item, Frame, Errors, UserLevel};
use crate::GetItem;

/// default RSCP Port
const DEFAULT_PORT: u16 = 5033;

/// RSCP Client object
pub struct Client {
    /// Connection status
    pub connected: bool,

    /// the encryption and decryption processor
    enc_processor: RscpEncryption,

    /// the connection stream as mutex
    connection: Option<Arc<Mutex<TcpStream>>>,

    /// the username for connection
    username: String,

    /// password for connection
    password: String,
}

impl Client {
    /// returns RSCP Client
    /// 
    /// # Arguments
    /// 
    /// * `rscp_key` - RSCP encyption key
    /// * `username` - RSCP username
    /// * `password` - RSCP password
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rscp;
    /// let mut c = rscp::Client::new("RSCP_KEY", "RSCP_USER".to_string(), "RSCP_PASSWORD".to_string());
    /// ```
    pub fn new(rscp_key: &str, username: String, password: String) -> Self {
        Self {
            connected: false,
            connection: None,
            enc_processor: RscpEncryption::new(rscp_key),
            username: username,
            password
        }
    }

    /// Connects to given host
    /// 
    /// # Arguments
    /// 
    /// * `host` - Host addess of energy storage
    /// * `port` - Optional port, default 5033
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use rscp;
    /// let mut c = rscp::Client::new("RSCP_KEY", "RSCP_USER".to_string(), "RSCP_PASSWORD".to_string());
    /// match c.connect("energy.storage.local", None) {
    ///     Ok(_) => (),
    ///     Err(err) => {
    ///         panic!("Unable to connect: {:?}", err);#
    ///     }
    /// }
    /// ```
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
        match self.send_receive_frame(&frame) {
            Ok(result_frame) => { 
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

    /// Disconnects from host
    pub fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        self.connection.as_mut().unwrap().as_ref().lock().unwrap().shutdown(Shutdown::Both)?;
        Ok(())
    }

    /// Sends and receives frame from connection
    /// 
    /// # Arguments
    /// 
    /// * `frame` - frame to send
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use rscp;
    /// use rscp::GetItem;
    /// let mut c = rscp::Client::new("RSCP_KEY", "RSCP_USER".to_string(), "RSCP_PASSWORD".to_string());
    /// match c.connect("energy.storage.local", None) {
    ///     Ok(_) => (),
    ///     Err(err) => {
    ///         panic!("Unable to connect: {:?}", err);#
    ///     }
    /// }
    /// 
    /// let mut info_frame = rscp::Frame::new();
    /// info_frame.push_item(rscp::Item { tag: rscp::tags::INFO::SERIAL_NUMBER.into(), data: None } );
    /// 
    /// match c.send_receive_frame(&info_frame) {
    ///     Ok(result_frame) => {
    ///         println!("{}", result_frame.get_item_data::<String>(rscp::tags::INFO::SERIAL_NUMBER.into()).unwrap());
    ///     },
    ///     Err(err) => {
    ///         println!("Unable send: {:?}", err);
    ///     }
    /// }
    /// ```
    pub fn send_receive_frame(&mut self, frame: &Frame) -> Result<Frame> {
        debug!("<< {:?}", frame);
        let data = frame.to_bytes()?;
        // debug!("<< Frame: {:02x?}", data);
        let enc_data = self.enc_processor.encrypt(data)?;

        self.write_to_stream(&enc_data)?;
        let return_enc_data = self.read_from_stream()?;
        if return_enc_data.len() == 0 {
            bail!(Errors::ReceiveNothing)
        }

        let return_data = self.enc_processor.decrypt(return_enc_data)?;
        // debug!(">> Frame: {:02x?}", return_data);

        let result_frame = Frame::from_bytes(return_data)?;
        debug!(">> {:?}", result_frame);

        Ok(result_frame)
    }

    /// writes data to stream
    /// 
    /// # Arguments
    /// 
    /// * `data` - data to send
    fn write_to_stream(&mut self, data: &[u8]) -> Result<()> {
        if !self.connected {
            bail!(Errors::NotConnected)
        }
        self.connection.as_mut().unwrap().as_ref().lock().unwrap().write(&data)?;
        Ok(())
    }

    /// reads data from stream
    fn read_from_stream(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            bail!(Errors::NotConnected)
        }
        let mut buffer = [0 as u8; BLOCK_SIZE];
        let mut data: Vec<u8> = Vec::new();
        loop
        {
            match self.connection.as_mut().unwrap().as_ref().lock().unwrap().read_exact(&mut buffer) {
                Ok(_) => { data.extend_from_slice(&buffer); },
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => { break; },
                Err(_) => { break; } //return Err(anyhow!("error receiving data: {}", e))
            }
        };
        Ok(data)
    }
}
