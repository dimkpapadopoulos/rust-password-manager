use clearscreen;
use password_manager::{Entry, add, delete, get, input, list, load_from_file};
use std::{collections::HashMap, io::Write};

fn main() {
    let master_pwd = rpassword::prompt_password("Enter your master password: ")
        .expect("Failed to get master password.");
    clearscreen::clear().expect("failed to clear screen.");
    let mut vault: HashMap<String, Entry> = load_from_file("passwords.bin", &master_pwd);

    loop {
        print!(
            "This is the CLI password manager. Please type a command to continue \n(for a list of available commands type \"help\": "
        );
        let _ = std::io::stdout().flush();
        let command = input();
        match command.as_str() {
            "a" | "A" | "add" | "ADD" | "Add" | "1" => add(&mut vault, &master_pwd),

            "g" | "G" | "get" | "GET" | "Get" | "2" => get(&vault),

            "d" | "D" | "delete" | "DELETE" | "Delete" | "3" => delete(&mut vault, &master_pwd),

            "l" | "L" | "list" | "LIST" | "List" | "4" => list(&vault),

            "q" | "Q" | "quit" | "QUIT" | "Quit" | "5" => {
                println!("Now exiting the password manager.");
                break;
            }
            "h" | "H" | "help" | "HELP" | "Help" => {
                println!(
                    "List of available commands:
            1. add: Create a new entry in the password vault.
            2. get: Retrieve an entry from the password vault.
            3. delete: Delete an entry from the password vault.
            4. list: Lists all entries in the vault.
            5. quit: exit the program."
                );
            }
            _ => {
                println!("Unknown command.");
            }
        }
        println!("Press enter to go back to menu.");
        input();
        clearscreen::clear().expect("failed to clear screen");
    }
}
