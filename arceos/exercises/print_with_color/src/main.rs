#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::{colorln, println};

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    colorln!("[WithColor]: Hello, Arceos!");
    println!("[WithoutColor]: Hello, Arceos!");
}
