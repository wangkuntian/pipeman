use crate::common::args::DeployArgs;
use std::io::stdout;
use time::macros::{format_description, offset};
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::common::config::CONF;

pub fn init_logger(args: &DeployArgs) {
    let mut log_level = Level::INFO;
    if CONF::global().log.debug {
        log_level = Level::DEBUG;
    }
    let log_dir = format!(
        "{}/{}",
        CONF::global().default.work_dir,
        CONF::global().log.log_dir
    );
    let log_file = rolling::daily(log_dir.as_str(), "pipeman.log");
    let log_stdout = stdout.with_max_level(log_level);
    let time_fmt =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]");
    let timer = OffsetTime::new(offset!(+8), time_fmt);
    let format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true)
        .with_source_location(true)
        .with_ansi(false)
        .with_timer(timer)
        .compact();
    let subscriber = tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(log_level);

    if args.quiet {
        subscriber.with_writer(log_file).init();
    } else {
        subscriber.with_writer(log_stdout.and(log_file)).init();
    }
}
