use password_manager::{Entry, add, get, input, load_from_file};
use std::collections::HashMap;

fn main() {
    let mut vault: HashMap<String, Entry> = load_from_file("passwords.bin");

    loop {
        println!(
            "This is the CLI password manager. Please choose between the following actions:\n   1. add: Create a new entry in the password vault.\n   2. get: Retrieve an entry from the password vault.\n   3. quit: exit the program."
        );
        let command = input();
        match command.as_str() {
            "a" | "A" | "add" | "ADD" | "Add" | "1" => add(&mut vault),

            "g" | "G" | "get" | "GET" | "Get" | "2" => get(&vault),

            "q" | "Q" | "quit" | "QUIT" | "Quit" | "3" => {
                println!("Now exiting the password manager.");
                break;
            }

            _ => {
                println!("Unknown command. Try one of add, get or quit.");
            }
        }
    }
}
