extern crate afl;
extern crate roxmltree;

use std::str;

use afl::fuzz;

fn main() {
    fuzz(|data| {
        if let Ok(text) = str::from_utf8(data) {
            let _ = roxmltree::Document::parse(&text);
        }
    });
}
