use rand::RngCore;
use std::io::{Write, stdin, stdout};

pub fn input(str_to_print: &str) -> String {
    let mut input = String::new();
    _ = stdout().write_all(str_to_print.as_bytes());
    _ = stdout().flush();
    stdin().read_line(&mut input).expect("Failed to get input.");
    input.trim().to_string()
}
pub fn generate_password(len: usize) -> String {
    const CHARSET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789)(*&^%$#@!~";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.next_u32() as usize % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}
