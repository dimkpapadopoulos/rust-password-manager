use std::fs::File;
use std::io::Write;

fn main() {
    let filename = "large_test.csv";
    println!("Generating 2,000 entries into '{}'...", filename);

    let mut file = File::create(filename).expect("Failed to create file");

    // 1. Write the Header (4 fields)
    writeln!(file, "name,url,username,password").expect("Failed to write header");

    // 2. Loop to generate data
    for i in 0..2000 {
        // Every 100th line: Create a "Missing Password" error (Only 3 columns)
        if i % 100 == 0 && i != 0 {
            writeln!(
                file,
                "Corrupt_Service_{},http://error.com,user_forgot_pass",
                i
            )
            .unwrap();
            println!("   -> Injected broken row at line {}", i + 2); // +2 for header & 1-based indexing

        // Every 146th line: Create a "Garbage" error (Only 1 column)
        } else if i % 146 == 0 && i != 0 {
            writeln!(file, "Just_Random_Garbage_Text_{}", i).unwrap();
            println!("   -> Injected garbage row at line {}", i + 2);

        // Normal Valid Row (4 Columns)
        } else {
            writeln!(
                file,
                "Service_{},http://site{}.com,user_{},secure_pass_{}",
                i, i, i, i
            )
            .unwrap();
        }
    }

    println!("Done! File created.");
}
