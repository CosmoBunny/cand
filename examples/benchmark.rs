use std::time::Instant;

use cand::Logger;

fn main() {
    let time = Instant::now();
    // Time visible true according Instant. () for normal printing.
    let mut logger = Logger(Instant::now(), ());

    for i in 1..=10_000 {
        logger.log_info(format!("Logger no: {}", i));
    }

    logger.log_ok(format_args!("Time taken in {:?}", time.elapsed()));
}
