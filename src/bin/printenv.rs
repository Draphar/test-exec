//! `printenv` fallback.
//! Only UTF-8 since converting `OsStr` to bytes is painful.

use std::env::{args, var, vars};

fn main() {
    if let Some(variable) = args().nth(1) {
        println!("{}", var(variable).unwrap_or_default());
    }else{
        for i in vars(){
            println!("{}={}", i.0, i.1);
        };
    }
}