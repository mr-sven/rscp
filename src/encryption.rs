use anyhow::{anyhow, Result};
use simple_rijndael::Errors;
use simple_rijndael::impls::RijndaelCbc;
use simple_rijndael::paddings::ZeroPadding;

pub const BLOCK_SIZE: usize = 32;

pub struct RscpEncryption {
    key: [u8; BLOCK_SIZE],
    enc_iv: [u8; BLOCK_SIZE],
    dec_iv: [u8; BLOCK_SIZE]
}

impl RscpEncryption {
    pub fn new(rscp_key: &str) -> Self {
        
        let rscp_key_bytes = rscp_key.as_bytes();
        let mut key = [0xff; BLOCK_SIZE];

        let len = if rscp_key_bytes.len() >= BLOCK_SIZE {
            BLOCK_SIZE
        } else {
            rscp_key_bytes.len()
        };
    
        key[..len].clone_from_slice(&rscp_key_bytes);

        Self {
            key: key,
            dec_iv: [0xff; BLOCK_SIZE],
            enc_iv: [0xff; BLOCK_SIZE]
        }
    }

    pub fn encrypt(&mut self, data: Vec<u8>) -> Result<Vec<u8>> {
    
        // encrypt the data using key an enc iv
        let result = RijndaelCbc::<ZeroPadding>::new(&self.key, BLOCK_SIZE)
            .map_err(|error| to_anyhow(error))?
            .encrypt(&self.enc_iv, data)
            .map_err(|error| to_anyhow(error))?;

        // store enc iv back for next encryption
        self.enc_iv.clone_from_slice(&result[result.len() - 32..]);

        Ok(result)
    }

    pub fn decrypt(&mut self, data: Vec<u8>) -> Result<Vec<u8>> {

        // decrypt the data using key an enc iv
        let result = RijndaelCbc::<ZeroPadding>::new(&self.key, BLOCK_SIZE)
            .map_err(|error| to_anyhow(error))?
            .decrypt(&self.dec_iv, data.to_vec())
            .map_err(|error| to_anyhow(error))?;

        // store enc iv back for next encryption
        self.dec_iv.clone_from_slice(&data[data.len() - 32..]);

        Ok(result)
    }
}

fn to_anyhow(error: Errors) -> anyhow::Error {
    match error {
        Errors::InvalidDataSize => anyhow!("InvalidDataSize"),
        Errors::InvalidBlockSize => anyhow!("InvalidBlockSize"),
        Errors::InvalidKeySize => anyhow!("InvalidKeySize"),
    }
}