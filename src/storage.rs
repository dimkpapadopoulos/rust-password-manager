use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
// Crypto Imports
use argon2::Argon2;
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use rand::rngs::OsRng;

// Import our Entry struct from the sibling module
use crate::models::Entry;

fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];

    let argon2 = Argon2::default();
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Failed to derive key.");
    key
}

pub fn save_to_file(vault: &HashMap<String, Entry>, filename: &str, master_pwd: &str) {
    let mut file = File::create(filename).expect("Could not write file to disk.");
    let data_bytes = bincode::serialize(vault).expect("Failed to serialize data.");
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let key = derive_key(master_pwd, &salt);
    let cipher = ChaCha20Poly1305::new_from_slice(&key).expect("Invalid key length.");
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let encrypted_data = cipher
        .encrypt(nonce, data_bytes.as_ref())
        .expect("Failed to encrypt data.");
    file.write_all(&salt).expect("Failed to write salt");
    file.write_all(&nonce_bytes).expect("Failed to write nonce");
    file.write_all(&encrypted_data)
        .expect("Failed to write encrypted data");
}

pub fn load_from_file(filename: &str, master_pwd: &str) -> HashMap<String, Entry> {
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };

    let mut salt = [0u8; 16];
    match file.read_exact(&mut salt) {
        Ok(_) => {}
        Err(_) => return HashMap::new(),
    }

    let key = derive_key(master_pwd, &salt);

    let mut nonce_bytes = [0u8; 12];
    file.read_exact(&mut nonce_bytes)
        .expect("Failed to read nonce");
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .expect("Failed to read ciphertext");

    let cipher = ChaCha20Poly1305::new_from_slice(&key).expect("Invalid Key");
    let decrypted_bytes = match cipher.decrypt(nonce, encrypted_data.as_ref()) {
        Ok(bytes) => bytes,
        Err(_) => panic!("Incorrect Password or Corrupted Data!"),
    };

    bincode::deserialize(&decrypted_bytes).expect("Failed to load data")
}
