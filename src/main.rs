//Modules
mod commands;
mod models;
mod storage;
mod utils;
//Bring crates into scope
use crate::commands::*;
use crate::models::Entry;
use crate::storage::load_from_file;
use crate::utils::input;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use secrecy::ExposeSecret;
use std::collections::HashMap;
use std::io::stdout;

fn main() {
    let banner = "[a]dd [g]et [l]ist [e]dit [d]elete [s]earch [n]ew [i]mport [q]uit";
    let master_pwd = secrecy::Secret::new(
        rpassword::prompt_password("Enter your master password: ")
            .expect("Failed to get master password."),
    );
    execute!(stdout(), Clear(ClearType::All)).expect("failed to clear screen.");
    let mut vault: HashMap<String, Entry> =
        load_from_file("passwords.bin", &master_pwd.expose_secret());

    loop {
        println!("\n{}", banner);

        let raw = input("> ");
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        match cmd.as_str() {
            "add" | "a" => add(&mut vault, &master_pwd.expose_secret(), &args),
            "get" | "g" => get(&vault, &args),
            "delete" | "d" => delete(&mut vault, &master_pwd.expose_secret(), &args),
            "list" | "l" => list(&vault, &args),
            "gen" | "n" => {
                generate(&args);
            }
            "edit" | "e" => edit(&mut vault, &master_pwd.expose_secret(), &args),
            "search" | "s" => search(&vault, &args),
            "import" | "i" => import(&mut vault, &master_pwd, &args),
            "quit" | "q" => {
                println!("Now exiting the password manager.");
                break;
            }
            "help" | "h" => println!("{}", banner),
            _ => println!("{}", banner),
        }
    }
}
