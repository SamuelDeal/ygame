use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Verbosity {
	Critical,
	Error,
	Warning,
	Info,
	Debug,
}

impl From<u8> for Verbosity {
	fn from(value: u8) -> Verbosity {
		match value {
			0 => Verbosity::Critical,
			1 => Verbosity::Error,
			2 => Verbosity::Warning,
			3 => Verbosity::Info,
			_ => Verbosity::Debug,
		}
	}
}

impl From<Verbosity> for u8 {
	fn from(value: Verbosity) -> u8 {
		match value {
			Verbosity::Critical => 0,
			Verbosity::Error => 1,
			Verbosity::Warning => 2,
			Verbosity::Info => 3,
			Verbosity::Debug => 4,
		}
	}
}

pub fn set_verbosity(lvl: Verbosity) {
	VERBOSITY_LEVEL.store(lvl.into(), Ordering::SeqCst)
}

pub fn log_msg(lvl: Verbosity, msg: &str) {
	if VERBOSITY_LEVEL.load(Ordering::SeqCst) >= lvl.into() {
		if lvl <= Verbosity::Warning {
			eprintln!("{}", msg);
		} else {
			println!("{}", msg);
		}
	}
}

pub fn log_err(err: &dyn std::error::Error) {
	error!("{}", err)
}

static VERBOSITY_LEVEL: AtomicU8 = AtomicU8::new(1); //Verbosity::Error

#[macro_export]
macro_rules! critical {
    ($($t:tt)*) => (
        crate::log::log_msg(crate::log::Verbosity::Critical, &format_args!($($t)*).to_string());
    )
}

#[macro_export]
macro_rules! error {
    ($($t:tt)*) => (
        crate::log::log_msg(crate::log::Verbosity::Error, &format_args!($($t)*).to_string());
    )
}

#[macro_export]
macro_rules! warning {
    ($($t:tt)*) => (
        crate::log::log_msg(crate::log::Verbosity::Warning, &format_args!($($t)*).to_string());
    )
}

#[macro_export]
macro_rules! info {
    ($($t:tt)*) => (
        crate::log::log_msg(crate::log::Verbosity::Info, &format_args!($($t)*).to_string());
    )
}

#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => (
        crate::log::log_msg(crate::log::Verbosity::Debug, &format_args!($($t)*).to_string());
    )
}

pub(crate) use critical;
pub(crate) use debug;
pub(crate) use error;
pub(crate) use info;
pub(crate) use warning;
