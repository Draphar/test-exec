//! `sleep` fallback.

use std::env::args;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .map(|a| Duration::from_secs(a))
        .map(|a| sleep(a))
        .unwrap();
}