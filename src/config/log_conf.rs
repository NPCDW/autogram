use std::str::FromStr;

use tracing_subscriber::{filter::{self, LevelFilter}, Layer, prelude::*, fmt::time::OffsetTime};

pub fn init() {
    let level = match std::env::var("LOG_LEVEL") {
        Ok(s) => s,
        Err(_) => "info".to_string(),
    };
    let local_time = OffsetTime::new(
        time::UtcOffset::from_hms(8, 0, 0).unwrap(),
        time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]").unwrap(),
    );
    let filter = filter::Targets::new()
        .with_target("tdlib", LevelFilter::ERROR)
        .with_target("", LevelFilter::from_str(&level).unwrap());
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_writer(std::io::stderr)
                .with_timer(local_time.clone())
                .with_filter(filter)
        )
        .init();
}
