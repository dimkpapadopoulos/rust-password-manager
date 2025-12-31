use bincode;
use rpassword;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

pub fn input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to get input.");
    let trimmed = input.trim();
    trimmed.to_string()
}

pub fn save_to_file(vault: &HashMap<String, Entry>, filename: &str) {
    let file = File::create(filename).expect("Could not write file to disk.");
    bincode::serialize_into(file, vault).expect("Failed to save data.");
}

pub fn load_from_file(filename: &str) -> HashMap<String, Entry> {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => return HashMap::new(), // File not found? Return empty map immediately
    };

    // If we get here, the file exists, so we try to load the data
    bincode::deserialize_from(file).expect("Failed to load data")
}
pub fn add(vault: &mut HashMap<String, Entry>) {
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
    save_to_file(vault, "passwords.bin");
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
            println!("Press enter to return to main screen.");
            loop {
                input();
                break;
            }
        }
        None => {
            println!("No entry found under this name.\n")
        }
    }
}
pub fn list(vault: &HashMap<String, Entry>) {
    println!("Stored passwords: {}", vault.capacity());
    let mut i = 0;
    for (name, _entry) in vault {
        i += 1;
        println!("{}. {}", i, name);
    }
}
pub fn delete(vault: &mut HashMap<String, Entry>) {
    println!("Please enter the name of the entry you want to delete.");
    let name = input();
    let outcome = vault.remove(&name.clone());
    match outcome {
        Some(_) => {
            save_to_file(vault, "passwords.bin");
            println!("Entry deleted succesfully.");
        }
        None => println!("Entry not found in vault."),
    }
}
