//! Tells you when you press enter
//!
//! Run with:
//!
//! ```bash
//! cargo run --example enter
//! ```
//!
//! Press enter and watch the console. Press Ctrl+C to quit.

use keyboard::Key;

fn main() {
    println!("PRESS ENTER \n\n");
    loop {
        if Key::Enter.is_pressed() {
            println!("You pressed enter");
            break;
        }
    }
}