//! Helper crate for fast log initializing.
//!
//! This crate reexports all macros from [`log`] crate and uses [`env_logger`]
//! crate for logger initializing.
//!
//! Example:
//! ```rust
//! use logger::*;
//!
//! init_logger();
//!     
//! info!("Logger initialized successfully!");
//! ```

pub use log::{debug, error, info, trace, warn};

/// Default log level for debug compilations.
const DEBUG_LOG_LEVEL: &str = "trace";

/// Default log level for release compilations.
const RELEASE_LOG_LEVEL: &str = "info";

/// Environment variable for log level setting.
const LOG_LEVEL_ENV: &str = "LOG_LEVEL";

/// Environment variable for log style setting.
const LOG_STYLE_ENV: &str = "LOG_STYLE";

/// Default log style.
const LOG_STYLE: &str = "auto";

use env_logger::fmt::Formatter;
use log::Record;
use std::io;

/// Logger initializer. Run this function in binary crate to initialize logging.
/// This function should be ran only once.
pub fn init_logger() {
	let log_level = match cfg!(debug_assertions) {
		true => DEBUG_LOG_LEVEL,
		false => RELEASE_LOG_LEVEL,
	};

	env_logger::Builder::from_env(
		env_logger::Env::default()
			.filter_or(LOG_LEVEL_ENV, log_level)
			.write_style_or(LOG_STYLE_ENV, LOG_STYLE),
	)
	.format(format)
	.init();
}

/// Logging output format.
fn format(buf: &'_ mut Formatter, record: &'_ Record<'_>) -> io::Result<()> {
	use env_logger::fmt::Color;
	use log::Level;
	use std::io::Write;

	let mut error = buf.style();
	let mut warn = buf.style();
	let mut info = buf.style();
	let mut debug = buf.style();
	let mut trace = buf.style();

	error.set_color(Color::Red).set_bold(true);
	warn.set_color(Color::Yellow);
	info.set_color(Color::Cyan);
	debug.set_color(Color::Magenta);
	trace.set_color(Color::Blue);

	let level_style = match record.level() {
		Level::Error => error,
		Level::Warn => warn,
		Level::Info => info,
		Level::Debug => debug,
		Level::Trace => trace,
	};

	writeln!(
		buf,
		"{}\t{}",
		level_style.value(record.level()),
		record.args()
	)
}
