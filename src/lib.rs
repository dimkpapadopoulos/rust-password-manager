use arboard::Clipboard;
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
use std::io::{Read, Write, stdin, stdout};

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

pub fn input(str_to_print: &str) -> String {
    let mut input = String::new();
    _ = stdout().write_all(str_to_print.as_bytes());
    _ = stdout().flush();
    stdin().read_line(&mut input).expect("Failed to get input.");
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
    let name = input("Name: ");
    let url = input("Url: ");
    let username = input("Username: ");
    let password_input = rpassword::prompt_password("Password (leave empty for random password): ");
    if password_input.is_err() {
        return;
    }

    let mut password = password_input.unwrap();
    if password.is_empty() {
        password = generate();
    }
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
    let name = input("Name: ");
    println!(
        "Getting entry from vault. Press \"s\" to show password or \"c\" to copy it to clipboard."
    );
    let retrieved_entry = vault.get(&name);
    match retrieved_entry {
        Some(entry) => {
            println!(
                "Name: {}\nUrl: {}\nUsername: {}\nPassword: {}",
                entry.name,
                entry.url,
                entry.username,
                String::from_iter(std::iter::repeat_n("*", entry.password.len()))
            );
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if let Err(e) = clipboard.set_text(&entry.password) {
                        eprintln!("Failed to copy to clipboard: {}", e);
                    } else {
                        println!("Password copied to clipboard!");

                        // Optional: Clear clipboard after delay (requires spawning a thread)
                        // For now, let's just confirm it's copied.
                    }
                }
                Err(e) => eprintln!("Clipboard unavailable: {}", e),
            }
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
pub fn generate() -> String {
    let len_str = input("Length (default 16):");
    let len = len_str.parse::<usize>().unwrap_or(16); // Default to 16 if input is invalid

    let password = generate_password(len);
    println!("Generated Password: {}", password);

    // Auto-copy generated password too? Why not!
    match Clipboard::new() {
        Ok(mut clip) => {
            let _ = clip.set_text(&password);
            println!("Copied to clipboard!");
        }
        _ => {}
    }
    password
}

pub fn edit(vault: &mut HashMap<String, Entry>, master_pwd: &str) {
    let name = input("Entry to edit: ");

    if let Some(entry) = vault.get_mut(&name) {
        // .get_mut allows modification
        println!("Found entry for {}.", entry.name);

        let new_user = input("Enter new Username (press Enter to keep current): ");

        let new_pass = input("Enter new Password (press Enter to keep current):");

        if !new_user.is_empty() {
            entry.username = new_user;
        }

        if !new_pass.is_empty() {
            entry.password = new_pass;
        }

        save_to_file(&vault, "passwords.bin", &master_pwd);
        println!("Entry updated!");
    } else {
        println!("Service not found.");
    }
}

pub fn delete(vault: &mut HashMap<String, Entry>, master_pwd: &str) {
    let name = input("Name: ");
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

pub fn generate_password(len: usize) -> String {
    const CHARSET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789)(*&^%$#@!~";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.next_u32() as usize % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}
