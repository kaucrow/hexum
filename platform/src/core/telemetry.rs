use crate::prelude::*;
use crate::Config;
use tracing_subscriber::{
    fmt::{self, time::FormatTime, MakeWriter},
    layer::SubscriberExt,
    EnvFilter,
    Layer
};
use tracing_appender;
use chrono::{Datelike, Timelike};
use anyhow::Result;

/// A wrapper that strips ANSI escape codes from a writer
#[derive(Clone)]
struct AnsiStripper<W>(pub W);

impl<'a, W> MakeWriter<'a> for AnsiStripper<W>
where
    W: MakeWriter<'a>,
{
    type Writer = strip_ansi_escapes::Writer<W::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        strip_ansi_escapes::Writer::new(self.0.make_writer())
    }
}

/// Log timestamp formatter, with the format `[day-month-year] [hour:minute:second.nanosecond]`.
#[derive(Clone)]
struct TimeFormat;

impl FormatTime for TimeFormat {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();

        let (year, month, day, hour, minute, second, nano) =
            (now.year(), now.month(), now.day(),
             now.hour(), now.minute(), now.second(),
             now.timestamp_subsec_nanos());

        write!(w, "{}-{}-{} {:02}:{:02}:{:02}.{}", day, month, year, hour, minute, second, nano)
    }
}

/// Build a tracing subscriber.
pub async fn get_subscriber(config: &Config) -> Result<(impl tracing::Subscriber + Send + Sync, tracing_appender::non_blocking::WorkerGuard)> {
    let root_path = get_root_path();
    let log_dir = root_path.join("log");
    let log_path = log_dir.join("server.log");

    if log_path.exists() && log_path.is_file() {
        std::fs::remove_file(&log_path)?
    }

    let file_appender = tracing_appender::rolling::never(log_dir, "server.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter_str_debug = "debug,h2=info,actix_server=off,sqlx=warn,rumqttd=error,rumqttc=error";
    let filter_str_info = "info,h2=info,actix_server=off,sqlx=warn,rumqttd=error,rumqttc=error";

    let (console_filter, file_filter) = if config.debug {
        (filter_str_debug.to_string(), EnvFilter::new(filter_str_debug))
    } else {
        (filter_str_info.to_string(), EnvFilter::new(filter_str_info))
    };

    let console_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(console_filter));

    let subscriber = tracing_subscriber::Registry::default()
        .with(fmt::layer()
            .with_target(false)
            .with_writer(AnsiStripper(non_blocking))
            .with_timer(TimeFormat)
            .with_ansi(false)
            .with_ansi_sanitization(false)
            .with_filter(file_filter))
        .with(fmt::layer()
            .with_target(false)
            .with_writer(std::io::stdout)
            .with_timer(TimeFormat)
            .with_ansi(true)
            .with_ansi_sanitization(false)
            .with_filter(console_filter));

    Ok((subscriber, guard))
}

/// Set the tracing subscriber.
pub fn init(subscriber: impl tracing::Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}