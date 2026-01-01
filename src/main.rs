use clearscreen;
use password_manager::*;
use std::collections::HashMap;

fn main() {
    let master_pwd = rpassword::prompt_password("Enter your master password: ")
        .expect("Failed to get master password.");
    clearscreen::clear().expect("failed to clear screen.");
    let mut vault: HashMap<String, Entry> = load_from_file("passwords.bin", &master_pwd);

    loop {
        let command = input(
            "This is the CLI password manager. Please type a command to continue \n(for a list of available commands type \"help\" or \"h\"): ",
        );
        match command.as_str() {
            "add" | "ADD" | "Add" | "1" => add(&mut vault, &master_pwd),

            "get" | "GET" | "Get" | "2" => get(&vault),

            "delete" | "DELETE" | "Delete" | "3" => delete(&mut vault, &master_pwd),

            "list" | "LIST" | "List" | "4" => list(&vault),

            "gen" | "GEN" | "Gen" | "5" => _ = generate(),

            "edit" | "EDIT" | "Edit" | "6" => edit(&mut vault, &master_pwd),

            "q" | "Q" | "quit" | "QUIT" | "Quit" | "7" => {
                println!("Now exiting the password manager.");
                break;
            }
            "h" | "H" | "help" | "HELP" | "Help" => {
                println!(
                    "List of available commands (you can use either numbers or keywords, not case-sensitive):
            1. add: Create a new entry in the password vault.
            2. get: Retrieve an entry from the password vault.
            3. delete: Delete an entry from the password vault.
            4. list: Lists all entries in the vault.
            5. gen: Generate a random password of a given length.
            6. edit: Change the attributes of an entry.
            7. quit: Exit the program."
                );
            }
            _ => {
                println!("Unknown command.");
            }
        }

        input("Press enter to go back to menu.");
        clearscreen::clear().expect("failed to clear screen");
    }
}
