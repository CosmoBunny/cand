use std::time::Instant;

use cand::MultiLogger;

fn main() {
    let mut logger = MultiLogger(Instant::now(), ());
    logger.clone().try_run_get(
        std::fs::read("examples/try_run_get.rs").map_err(|err| err.into()),
        |data| {
            logger.log_ok("File read Sucessfully");
            logger.try_run_get(
                String::from_utf8(data).map_err(|err| err.into()),
                |content| println!("{}", content),
            );
        },
    );
}
