use argon2::Argon2;
use bincode;
use chacha20poly1305::{
    ChaCha20Poly1305,
    Nonce,
    aead::{Aead, KeyInit}, // Traits for encryption
};
use rand::{RngCore, rngs::OsRng};
use rpassword;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

pub fn input() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to get input.");
    input.trim().to_string()
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
pub fn add(vault: &mut HashMap<String, Entry>, master_pwd: &str) {
    println!("Please enter the name of the entry.");
    let name = input();
    println!("Please enter the url of the entry.");
    let url = input();
    println!("Please enter the username for this entry.");
    let username = input();
    let password_input = rpassword::prompt_password("Your password: ");
    if password_input.is_err() {
        return;
    }
    let password = password_input.unwrap();

    println!("Adding entry to vault.");
    let new_entry = Entry {
        name,
        url,
        username,
        password,
    };
    let old_value = vault.insert(new_entry.name.clone(), new_entry);
    save_to_file(vault, "passwords.bin", master_pwd);
    match old_value {
        Some(_) => {
            println!("Entry updated.\n")
        }
        None => {
            println!("New entry added to vault.\n")
        }
    }
}
pub fn get(vault: &HashMap<String, Entry>) {
    println!("Please type the name of the entry you want to retrieve.");
    let name = input();
    println!("Getting entry from vault.");
    let retrieved_entry = vault.get(&name);
    match retrieved_entry {
        Some(entry) => {
            println!(
                "Name: {}\nUrl: {}\nUsername: {}\nPassword: {}",
                entry.name, entry.url, entry.username, entry.password
            );
        }
        None => {
            println!("No entry found under this name.\n")
        }
    }
}
pub fn list(vault: &HashMap<String, Entry>) {
    println!("Stored passwords: {}", vault.len());
    let mut i = 0;
    for (name, _entry) in vault {
        i += 1;
        println!("{}. {}\n", i, name);
    }
}
pub fn delete(vault: &mut HashMap<String, Entry>, master_pwd: &str) {
    println!("Please enter the name of the entry you want to delete.");
    let name = input();
    let outcome = vault.remove(&name.clone());
    match outcome {
        Some(_) => {
            save_to_file(vault, "passwords.bin", &master_pwd);
            println!("Entry deleted succesfully.");
        }
        None => println!("Entry not found in vault."),
    }
}

fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];

    let argon2 = Argon2::default();
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Failed to derive key.");
    key
}
