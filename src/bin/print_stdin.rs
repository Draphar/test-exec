//! Copy `stdin` to `stdout`.

use std::process::exit;
use std::io::{copy, stdin, stdout};

fn main() {
    let status = match copy(&mut stdin().lock(), &mut stdout().lock()) {
        Ok(_) => 0,
        Err(_) => 1
    };
    exit(status)
}
