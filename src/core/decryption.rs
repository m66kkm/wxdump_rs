// src/core/decryption.rs

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use anyhow::Result; 

// Cryptography crates
use aes::Aes256;
use aes::cipher::KeyIvInit; 
use aes::cipher::generic_array::GenericArray; 
use aes::cipher::generic_array::typenum::{U16, Unsigned}; 
use hmac::{Hmac, Mac};
use hmac::digest::FixedOutput; 
use sha1::Sha1;
use pbkdf2::pbkdf2_hmac;
use cbc::cipher::BlockDecryptMut; 

type AesBlock = GenericArray<u8, U16>;

const SQLITE_FILE_HEADER: &[u8] = b"SQLite format 3\x00";
const KEY_SIZE: usize = 32; 
const DEFAULT_PAGESIZE: usize = 4096;
const SALT_SIZE: usize = 16;
const IV_SIZE: usize = 16;
const HMAC_SHA1_SIZE: usize = 20; 
const RESERVED_SIZE: usize = 48; 

type HmacSha1 = Hmac<Sha1>; // This alias is now used

#[derive(Debug)]
pub enum DecryptionError {
    Io(std::io::Error),
    FileTooShort,
    HmacVerificationFailed,
    KeyDerivationFailed, // Still unused, but keeping for now
    HexDecodingFailed(hex::FromHexError),
    Other(String),
}

impl From<std::io::Error> for DecryptionError {
    fn from(err: std::io::Error) -> DecryptionError {
        DecryptionError::Io(err)
    }
}
impl From<hex::FromHexError> for DecryptionError {
    fn from(err: hex::FromHexError) -> DecryptionError {
        DecryptionError::HexDecodingFailed(err)
    }
}

impl std::fmt::Display for DecryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecryptionError::Io(e) => write!(f, "IO error: {}", e),
            DecryptionError::FileTooShort => write!(f, "File is too short to be a valid encrypted database"),
            DecryptionError::HmacVerificationFailed => write!(f, "HMAC verification failed, incorrect key or corrupted file"),
            DecryptionError::KeyDerivationFailed => write!(f, "Key derivation failed"),
            DecryptionError::HexDecodingFailed(e) => write!(f, "Hex decoding failed: {}", e),
            DecryptionError::Other(s) => write!(f, "Decryption error: {}", s),
        }
    }
}
impl std::error::Error for DecryptionError {}


pub fn decrypt_database_file(
    encrypted_db_path: &Path,
    output_path: &Path,
    key_hex: &str,
) -> Result<(), DecryptionError> { 
    if !encrypted_db_path.exists() || !encrypted_db_path.is_file() {
        return Err(DecryptionError::Other(format!("Encrypted DB file not found: {:?}", encrypted_db_path)));
    }
    if key_hex.len() != 64 {
        return Err(DecryptionError::Other("Key hex string must be 64 characters long.".to_string()));
    }

    let password_bytes = hex::decode(key_hex).map_err(DecryptionError::from)?;

    let mut encrypted_file = File::open(encrypted_db_path)?;
    let mut encrypted_data = Vec::new();
    encrypted_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < DEFAULT_PAGESIZE {
        return Err(DecryptionError::FileTooShort);
    }

    let salt = &encrypted_data[0..SALT_SIZE];
    let mut aes_key_arr = [0u8; KEY_SIZE]; // Renamed to avoid conflict if KEY_SIZE was a type
    pbkdf2_hmac::<Sha1>(&password_bytes, salt, 64000, &mut aes_key_arr);
    
    let mac_salt_array: [u8; SALT_SIZE] = core::array::from_fn(|i| salt[i] ^ 0x3A);
    let mut hmac_key_material = [0u8; KEY_SIZE]; 
    pbkdf2_hmac::<Sha1>(&aes_key_arr, &mac_salt_array, 2, &mut hmac_key_material);

    let first_page_data_for_hmac = &encrypted_data[SALT_SIZE..(DEFAULT_PAGESIZE - RESERVED_SIZE + IV_SIZE)]; 
    let stored_hmac = &encrypted_data[(DEFAULT_PAGESIZE - RESERVED_SIZE + IV_SIZE)..(DEFAULT_PAGESIZE - RESERVED_SIZE + IV_SIZE + HMAC_SHA1_SIZE)]; 

    let mut mac = HmacSha1::new_from_slice(&hmac_key_material) // Using HmacSha1 type alias
        .map_err(|e| DecryptionError::Other(format!("Failed to create HMAC-SHA1 instance: {}",e)))?;
    mac.update(first_page_data_for_hmac);
    mac.update(&1u32.to_le_bytes()); 

    let calculated_hmac_bytes = mac.finalize_fixed();
    if calculated_hmac_bytes.as_slice() != stored_hmac {
         println!("[Decryption] Calculated HMAC: {:02x?}", calculated_hmac_bytes.as_slice());
         println!("[Decryption] Stored HMAC: {:02x?}", stored_hmac);
        return Err(DecryptionError::HmacVerificationFailed);
    }
    println!("[Decryption] HMAC for the first page verified successfully.");

    let mut decrypted_writer = File::create(output_path)?;
    decrypted_writer.write_all(SQLITE_FILE_HEADER)?; // Using SQLITE_FILE_HEADER const

    let num_pages = encrypted_data.len() / DEFAULT_PAGESIZE; // Using DEFAULT_PAGESIZE const
    const AES_BLOCK_SIZE_USIZE_CONST: usize = U16::USIZE; // Using U16::USIZE

    for i in 0..num_pages {
        let page_offset = i * DEFAULT_PAGESIZE;
        let page_end = page_offset + DEFAULT_PAGESIZE;
        if page_end > encrypted_data.len() { break; } 
        let page_slice = &encrypted_data[page_offset..page_end];

        let data_to_decrypt: &[u8];
        let iv_slice: &[u8]; 

        if i == 0 { 
            data_to_decrypt = &page_slice[SALT_SIZE..(DEFAULT_PAGESIZE - RESERVED_SIZE)]; 
            iv_slice = &page_slice[(DEFAULT_PAGESIZE - RESERVED_SIZE)..(DEFAULT_PAGESIZE - RESERVED_SIZE + IV_SIZE)];
        } else { 
            data_to_decrypt = &page_slice[0..(DEFAULT_PAGESIZE - RESERVED_SIZE)]; 
            iv_slice = &page_slice[(DEFAULT_PAGESIZE - RESERVED_SIZE)..(DEFAULT_PAGESIZE - RESERVED_SIZE + IV_SIZE)];
        }
        
        if data_to_decrypt.is_empty() { continue; }
        
        if data_to_decrypt.len() % AES_BLOCK_SIZE_USIZE_CONST != 0 { 
            return Err(DecryptionError::Other(format!("Data to decrypt for page {} is not a multiple of AES block size ({} bytes): length {}", i, AES_BLOCK_SIZE_USIZE_CONST, data_to_decrypt.len())));
        }
        
        let mut buffer = data_to_decrypt.to_vec(); 
        
        let key_ga = GenericArray::from_slice(&aes_key_arr);
        let iv_ga = GenericArray::from_slice(iv_slice);
        let mut cipher = cbc::Decryptor::<Aes256>::new(key_ga, iv_ga);
        
        for chunk in buffer.chunks_exact_mut(AES_BLOCK_SIZE_USIZE_CONST) {
            let block = AesBlock::from_mut_slice(chunk); // Corrected to use AesBlock type alias
            cipher.decrypt_block_mut(block);
        }
        
        decrypted_writer.write_all(&buffer)?;

        // Write back the original reserved 48 bytes from the encrypted page, like Python does
        // page_slice is the full current encrypted page.
        // The last 48 bytes are page_slice[(DEFAULT_PAGESIZE - RESERVED_SIZE)..]
        decrypted_writer.write_all(&page_slice[(DEFAULT_PAGESIZE - RESERVED_SIZE)..])?;
    }
    
    println!("[Decryption] Database (with original reserved areas) decrypted successfully to {:?}", output_path);
    Ok(())
}