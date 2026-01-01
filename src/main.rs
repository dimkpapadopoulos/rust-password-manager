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
use clearscreen;
use secrecy::ExposeSecret;
use std::collections::HashMap;

fn main() {
    let help =
        "List of available commands (you can use either numbers or keywords, not case-sensitive):
1. add: Create a new entry in the password vault.
2. get: Retrieve an entry from the password vault.
3. delete: Delete an entry from the password vault.
4. list: Lists all entries in the vault.
5. gen: Generate a random password of a given length.
6. edit: Change the attributes of an entry.
7. import: Import information from a csv file.
quit: Exit the program.
help: This menu.";
    let master_pwd = secrecy::Secret::new(
        rpassword::prompt_password("Enter your master password: ")
            .expect("Failed to get master password."),
    );
    clearscreen::clear().expect("failed to clear screen.");
    let mut vault: HashMap<String, Entry> =
        load_from_file("passwords.bin", &master_pwd.expose_secret());

    loop {
        let command = input(
            "This is the CLI password manager. Please type a command to continue \n(for a list of available commands type \"help\" or \"h\"): ",
        );
        match command.to_lowercase().as_str() {
            "add" | "1" => add(&mut vault, &master_pwd.expose_secret()),
            "get" | "2" => get(&vault),
            "delete" | "3" => delete(&mut vault, &master_pwd.expose_secret()),
            "list" | "4" => list(&vault),
            "gen" | "5" => _ = generate(),
            "edit" | "6" => edit(&mut vault, &master_pwd.expose_secret()),
            "search" | "7" => search(&vault),
            "import" | "8" => import(&mut vault, &master_pwd),
            "q" | "quit" => {
                println!("Now exiting the password manager.");
                break;
            }
            "h" | "help" => {
                println!("{help}");
            }
            _ => {
                println!("{help}");
            }
        }

        input("Press enter to go back to menu.");
        clearscreen::clear().expect("failed to clear screen");
    }
}
