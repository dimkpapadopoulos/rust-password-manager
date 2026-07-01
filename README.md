## Rust CLI Password Manager

A CLI Password manager written in Rust. It's fast, it's secure and it's very barebones.

### How to use

After downloading, open a terminal in the project's root directory and use *cargo run --bin password_manager* .
Input your master password. This is what you will use to unlock the manager from now on. Make sure you keep it somewhere safe.

Use the following commands (or numbers) to navigate the different functions:

1) **Add**: Insert a new password into the manager's memory. It saves a name, a URL and the password itself. The last can be randomly generated.
2) **Get**: Retrieve an entry from memory. It shows the name, URL but obfuscates the password with asterisks until you press "S" on your keyboard. You can also copy it to clipboard with "C" (expires after 1 minute).
3) **Delete**: Delete an entry from memory.
4) **List**: Lists all entries by name (no passwords here). Helps you find what to input in **Get**.
5) Generate: Randomly generate a password of a given length. It gets copied to clipboard automatically for 1 minute.
6) Edit: Change any attribute of an entry. Use its name to designate which one.
7) Search: Input a search term to find entry names that include it.
8) Import: batch import passwords from a csv file. It should be formatted like this: [name,url,username,password].
