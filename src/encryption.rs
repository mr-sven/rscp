use anyhow::Result;
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
    
        key[..len].clone_from_slice(&rscp_key_bytes[..len]);

        Self {
            key: key,
            dec_iv: [0xff; BLOCK_SIZE],
            enc_iv: [0xff; BLOCK_SIZE]
        }
    }

    pub fn encrypt(&mut self, data: Vec<u8>) -> Result<Vec<u8>> {
    
        // encrypt the data using key an enc iv
        let result = RijndaelCbc::<ZeroPadding>::new(&self.key, BLOCK_SIZE)?.encrypt(&self.enc_iv, data)?;

        // store enc iv back for next encryption
        self.enc_iv.clone_from_slice(&result[result.len() - 32..]);

        Ok(result)
    }

    pub fn decrypt(&mut self, data: Vec<u8>) -> Result<Vec<u8>> {

        // decrypt the data using key an enc iv
        let result = RijndaelCbc::<ZeroPadding>::new(&self.key, BLOCK_SIZE)?.decrypt(&self.dec_iv, data.to_vec())?;

        // store enc iv back for next encryption
        self.dec_iv.clone_from_slice(&data[data.len() - 32..]);

        Ok(result)
    }
}

/// ################################################
///      TEST TEST TEST
/// ################################################

#[test]
fn test_key_size() {
    let enc = RscpEncryption::new("00011122233344455566677788899900");
    assert_eq!(enc.key.len(), BLOCK_SIZE);

    let enc2 = RscpEncryption::new("000111222333444555666777888999000");
    assert_eq!(enc2.key.len(), BLOCK_SIZE);
}

#[test]
fn test_encryption() {
    let mut enc = RscpEncryption::new("RSCP_KEY");

    let mut res = enc.encrypt("00011122233344455566677788899900".as_bytes().to_vec()).unwrap();
    assert_eq!(res, vec![
        0x8d, 0xfa, 0xc7, 0x4d, 0xcb, 0x33, 0x0b, 0x0d, 0x23, 0xe3, 0x4e, 0xfd, 0xe4, 0x28, 0xcb, 0xcd, 
        0x9b, 0x3d, 0x8c, 0xe9, 0x2a, 0xc5, 0x3a, 0x26, 0xf1, 0x17, 0x41, 0x87, 0xa7, 0x1a, 0x48, 0xca], "Test 32 byte encryption");

    res = enc.encrypt("000111222333444555666777888999000".as_bytes().to_vec()).unwrap();
    assert_eq!(res, vec![
        0xc0, 0x50, 0x27, 0xbb, 0xd6, 0x0d, 0xf4, 0xa3, 0xc1, 0x98, 0xd9, 0xee, 0x2d, 0xa9, 0xf3, 0xf6, 
        0x34, 0x04, 0x76, 0x5b, 0xce, 0x0b, 0x12, 0xa9, 0x9d, 0x43, 0x87, 0x8b, 0x78, 0xe8, 0xee, 0x33, 
        0x6c, 0xbc, 0x00, 0x44, 0xcf, 0xe2, 0x86, 0x94, 0xf1, 0xde, 0x9e, 0x47, 0x24, 0xe5, 0xab, 0x59, 
        0x8f, 0x64, 0x0f, 0xf4, 0x19, 0x62, 0x82, 0x84, 0x34, 0xe2, 0x00, 0x9a, 0xcc, 0x13, 0x89, 0xfd], "Test 33 byte encryption");
}

#[test]
fn test_decryption() {
    let mut enc = RscpEncryption::new("RSCP_KEY");

    let mut res = enc.decrypt(vec![
        0x8d, 0xfa, 0xc7, 0x4d, 0xcb, 0x33, 0x0b, 0x0d, 0x23, 0xe3, 0x4e, 0xfd, 0xe4, 0x28, 0xcb, 0xcd, 
        0x9b, 0x3d, 0x8c, 0xe9, 0x2a, 0xc5, 0x3a, 0x26, 0xf1, 0x17, 0x41, 0x87, 0xa7, 0x1a, 0x48, 0xca]).unwrap();
    assert_eq!(res, "00011122233344455566677788899900".as_bytes(), "Test 32 byte decryption");

    res = enc.decrypt(vec![
        0xc0, 0x50, 0x27, 0xbb, 0xd6, 0x0d, 0xf4, 0xa3, 0xc1, 0x98, 0xd9, 0xee, 0x2d, 0xa9, 0xf3, 0xf6, 
        0x34, 0x04, 0x76, 0x5b, 0xce, 0x0b, 0x12, 0xa9, 0x9d, 0x43, 0x87, 0x8b, 0x78, 0xe8, 0xee, 0x33, 
        0x6c, 0xbc, 0x00, 0x44, 0xcf, 0xe2, 0x86, 0x94, 0xf1, 0xde, 0x9e, 0x47, 0x24, 0xe5, 0xab, 0x59, 
        0x8f, 0x64, 0x0f, 0xf4, 0x19, 0x62, 0x82, 0x84, 0x34, 0xe2, 0x00, 0x9a, 0xcc, 0x13, 0x89, 0xfd]).unwrap();
    assert_eq!(res, "000111222333444555666777888999000\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes(), "Test 33 byte decryption");
}