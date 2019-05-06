//! `pwd` fallback.

use std::env::current_dir;

fn main() {
    println!("{}", current_dir().unwrap().display());
}