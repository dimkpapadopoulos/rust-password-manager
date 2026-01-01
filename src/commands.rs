use crate::models::Entry;
use crate::storage::save_to_file;
use crate::utils::{generate_password, input};
use arboard::Clipboard;
use rpassword::prompt_password;
use secrecy::{ExposeSecret, Secret};
use std::collections::HashMap;
use std::fs::File;
use std::{thread, time};

const DB_FILE: &str = "passwords.bin";

pub fn add(vault: &mut HashMap<String, Entry>, master_pwd: &str) {
    let name = input("Name: ");
    let url = input("Url: ");
    let username = input("Username: ");
    let password_input = prompt_password("Password (leave empty for random password): ");
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
        password: Secret::new(password),
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
            let pass_to_copy = entry.password.expose_secret().clone();
            println!(
                "Name: {}\nUrl: {}\nUsername: {}\nPassword: {}",
                entry.name,
                entry.url,
                entry.username,
                String::from_iter(std::iter::repeat_n("*", pass_to_copy.len()))
            );
            match Clipboard::new() {
                Ok(mut clip) => {
                    if clip.set_text(pass_to_copy).is_ok() {
                        println!("Password copied to clipboard! (Clearing in a minute...)");
                        thread::spawn(move || {
                            thread::sleep(time::Duration::from_secs(60));
                            if let Ok(mut c) = Clipboard::new() {
                                let _ = c.set_text("");
                            }
                        });
                    } else {
                        eprintln!("Failed to copy to clipboard.");
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
        println!("{}. {}", i, name);
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

        let password_input =
            rpassword::prompt_password("Enter new Password (press Enter to keep current): ");
        if password_input.is_err() {
            return;
        }

        let new_pass = password_input.unwrap();

        if !new_user.is_empty() {
            entry.username = new_user;
        }

        if !new_pass.is_empty() {
            entry.password = Secret::new(new_pass);
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

pub fn search(vault: &HashMap<String, Entry>) {
    let query = input("Search query: ");
    println!("--- Matches ---");
    for key in vault.keys() {
        if key.to_lowercase().contains(&query.to_lowercase()) {
            println!(" - {}", key);
        }
    }
    println!("---------------");
}

pub fn import(vault: &mut HashMap<String, Entry>, master_pwd: &Secret<String>) {
    let path = input("Path of csv file: ");
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open file '{}': {}", path, e);
            return;
        }
    };
    let mut rdr = csv::Reader::from_reader(file);
    let mut count = 0;
    let mut skipped = 0;
    for result in rdr.records() {
        match result {
            Ok(record) => {
                if record.len() >= 4 {
                    let name = record[0].to_string();
                    let url = record[1].to_string();
                    let username = record[2].to_string();
                    let password = record[3].to_string();

                    // Create the Entry struct
                    let new_entry = Entry {
                        name: name.clone(),
                        url: url,
                        username,
                        password: Secret::new(password),
                    };

                    // Insert into Vault
                    vault.insert(name, new_entry);
                    count += 1;
                } else {
                    println!("Skipping row (not enough columns): {:?}", record);
                    skipped += 1;
                }
            }
            Err(_) => skipped += 1, // Skip corrupt rows
        }
    }

    // 4. Save the updated vault to disk immediately
    save_to_file(vault, DB_FILE, master_pwd.expose_secret());

    println!("--------------------------------------------------");
    println!("Import Complete!");
    println!("Added: {} entries", count);
    if skipped > 0 {
        println!("Skipped: {} invalid rows", skipped);
    }
    println!("--------------------------------------------------");
}
