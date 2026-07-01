use crate::models::Entry;
use crate::storage::save_to_file;
use crate::utils::{generate_password, input};
use arboard::Clipboard;
use crossterm::{
    cursor::{MoveToColumn, MoveUp},
    event::{Event, KeyCode, KeyEventKind, read},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use rpassword::prompt_password;
use secrecy::{ExposeSecret, Secret};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, stdout};

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

    match vault.get(&name) {
        Some(entry) => {
            let pass_secret = entry.password.expose_secret();
            let hidden_pass = "*".repeat(pass_secret.len());
            let mut is_visible = false;

            let mut stdout = stdout();
            let _ = enable_raw_mode();

            // Print initial state using \r\n for raw mode
            print!(
                "Name: {}\r\nUrl: {}\r\nUsername: {}\r\nPassword: {}\r\nPress \"s\" to toggle, \"c\" to copy, or Enter to exit.",
                entry.name, entry.url, entry.username, hidden_pass
            );
            let _ = stdout.flush();

            loop {
                if let Ok(Event::Key(event)) = read() {
                    // Ignore release events to prevent double-toggling
                    if event.kind != KeyEventKind::Press {
                        continue;
                    }

                    match event.code {
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            is_visible = !is_visible;
                            let display_pass = if is_visible {
                                pass_secret
                            } else {
                                &hidden_pass
                            };

                            // Move cursor to start of output, clear downwards, and redraw
                            let _ = execute!(
                                stdout,
                                MoveToColumn(0),
                                MoveUp(4),
                                Clear(ClearType::FromCursorDown)
                            );

                            print!(
                                "Name: {}\r\nUrl: {}\r\nUsername: {}\r\nPassword: {}\r\nPress \"s\" to toggle, \"c\" to copy, or Enter to exit.",
                                entry.name, entry.url, entry.username, display_pass
                            );
                            let _ = stdout.flush();
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            let _ = disable_raw_mode();
                            let pass_to_copy = pass_secret.clone();

                            match Clipboard::new() {
                                Ok(mut clip) => {
                                    if clip.set_text(pass_to_copy).is_ok() {
                                        println!("\r\nPassword copied! (Clearing in 60s...)");
                                        std::thread::spawn(move || {
                                            std::thread::sleep(std::time::Duration::from_secs(60));
                                            if let Ok(mut c) = Clipboard::new() {
                                                let _ = c.set_text("");
                                            }
                                        });
                                    } else {
                                        eprintln!("\r\nFailed to copy to clipboard.");
                                    }
                                }
                                Err(e) => eprintln!("\r\nClipboard error: {}", e),
                            }
                            break;
                        }
                        KeyCode::Enter => {
                            let _ = disable_raw_mode();
                            println!("\r");
                            break;
                        }
                        _ => continue,
                    }
                }
            }
            let _ = disable_raw_mode();
        }
        None => {
            println!("No entry found under this name.\n");
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

/// Parse a single CSV line, handling quoted fields (fields may contain commas inside quotes).
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
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
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut count = 0;
    let mut skipped = 0;

    // Skip the header line (matches csv::Reader's default has_headers=true behavior)
    let _header = lines.next();

    for result in lines {
        match result {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                let record = parse_csv_line(&line);
                if record.len() >= 4 {
                    let name = record[0].clone();
                    let url = record[1].clone();
                    let username = record[2].clone();
                    let password = record[3].clone();

                    vault.insert(
                        name.clone(),
                        Entry {
                            name,
                            url,
                            username,
                            password: Secret::new(password),
                        },
                    );
                    count += 1;
                } else {
                    println!("Skipping row (not enough columns): {:?}", record);
                    skipped += 1;
                }
            }
            Err(_) => skipped += 1,
        }
    }

    save_to_file(vault, DB_FILE, master_pwd.expose_secret());

    println!("--------------------------------------------------");
    println!("Import Complete!");
    println!("Added: {} entries", count);
    if skipped > 0 {
        println!("Skipped: {} invalid rows", skipped);
    }
    println!("--------------------------------------------------");
}
