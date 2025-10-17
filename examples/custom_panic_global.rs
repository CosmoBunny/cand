use std::{sync::Mutex, time::Instant};

use cand::{Logger, StorageProvider, TimeProvider, black_box_cand_global};
use once_cell::sync::Lazy;

static LOGGER: Lazy<Mutex<Logger<LocalTime, LogErrorStorage>>> = Lazy::new(|| {
    Mutex::new(Logger(
        LocalTime(Lazy::new(|| Instant::now())),
        LogErrorStorage {},
    ))
});

pub struct LocalTime(pub Lazy<Instant>);

impl TimeProvider for LocalTime {
    fn now() -> Self {
        LocalTime(Lazy::new(|| Instant::now()))
    }
    fn elapsed(&self) -> core::time::Duration {
        self.0.elapsed()
    }
    fn write(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.0)?;
        Ok(())
    }
}

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
    black_box_cand_global!(&LOGGER);

    panic!("Panic Testing");
}

#[derive(Default)]
struct LogErrorStorage;
