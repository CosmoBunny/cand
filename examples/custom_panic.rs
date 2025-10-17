use std::time::Instant;

use cand::{Logger, StorageProvider, black_box_cand};

impl StorageProvider for LogErrorStorage {
    fn write_data(&mut self, args: std::fmt::Arguments, _debuglevel: &cand::StatusLevel) {
        let formatted = args.to_string();
        let stripped = strip_ansi(&formatted);
        if let Err(err) = std::fs::write("log.txt", stripped) {
            println!("{} {}", formatted, err)
        }
        println!("{}", formatted)
    }
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut in_esc = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_esc = true;
        } else if in_esc {
            if c == 'm' {
                in_esc = false;
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn main() {
    println!("Hello, world!");

    black_box_cand!(Logger(Instant::now(), LogErrorStorage {}));

    panic!("Panic Testing");
}

#[derive(Default)]
struct LogErrorStorage;
