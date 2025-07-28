use std::time::Instant;

use cand::Logger;

fn main() {
    let logger = Logger(Instant::now(), ());
    let ((), mut logger) = logger.try_get(Err("Error line"), redirect);
    logger.log(cand::StatusLevel::Ok, "Run very well");
}

fn redirect(_logger: Logger<Instant, ()>) {
    println!("Emergency function")
}
