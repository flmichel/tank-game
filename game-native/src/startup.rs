use std::str::FromStr;

use tracing::{info, warn, Level};

pub fn setup_logger(level: &str) {
    let mut wrong_level = false;
    let log_level = Level::from_str(level)
        .map_err(|err| {
            wrong_level = true;
            err
        })
        .unwrap_or(Level::INFO);

    tracing_subscriber::fmt().with_max_level(log_level).init();
    info!("Logging initialized with {} as maximum level", log_level);
    if wrong_level {
        warn!("Log level \"{}\" is not a correct value", level);
    }
}
