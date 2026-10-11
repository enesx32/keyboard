//! Continuously prints every key that is currently held down.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example readout
//! ```
//!
//! Hold down a few keys at once and watch the console. Press Ctrl+C to quit.

use keyboard::pressed_keys;
use std::time::Duration;

fn main() {
    println!("Hold down some keys. Press Ctrl+C to quit.\n");

    loop {
        let keys = pressed_keys();
        if !keys.is_empty() {
            let names: Vec<String> = keys.iter().map(|k| format!("{k:?}")).collect();
            println!("{}", names.join(", "));
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}