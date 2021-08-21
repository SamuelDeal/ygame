use std::fmt::{Display, Formatter, Result};

use ygame_protocol as msg;

#[macro_export]
macro_rules! unimpl {
	($($t:tt)*) => {
		crate::error::Error::ImplError {
			details: format_args!($($t)*).to_string(),
			source:None,
			error_code: Some(msg::error::code::server::UNIMPL_ERROR)
		}
	};
}

#[derive(Debug)]
pub enum Error {
	ProtocolError {
		details: String,
		source: Option<Box<dyn std::error::Error>>,
		error_code: Option<u32>,
	},
	LobbyError {
		details: String,
		source: Option<Box<dyn std::error::Error>>,
		error_code: Option<u32>,
	},
	GameError {
		details: String,
		source: Option<Box<dyn std::error::Error>>,
		error_code: Option<u32>,
	},
	ServerError {
		details: String,
		source: Option<Box<dyn std::error::Error>>,
		error_code: Option<u32>,
	},
	ImplError {
		details: String,
		source: Option<Box<dyn std::error::Error>>,
		error_code: Option<u32>,
	},
}

impl From<crate::Result<()>> for crate::actor_msg::ResultMsg {
	fn from(value: crate::Result<()>) -> crate::actor_msg::ResultMsg {
		match value {
			Ok(_) => crate::actor_msg::ResultMsg(Ok(())),
			Err(e) => crate::actor_msg::ResultMsg(Err(format!("{}", e))),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::ProtocolError {
				details,
				source,
				error_code,
			} => format_error(
				f,
				error_code.unwrap_or(msg::error::code::protocol::PROTOCOL_ERROR),
				"Protocol error",
				details,
				source.as_deref(),
			),
			Error::ServerError {
				details,
				source,
				error_code,
			} => format_error(
				f,
				error_code.unwrap_or(msg::error::code::server::SERVER_ERROR),
				"Server error",
				details,
				source.as_deref(),
			),
			Error::ImplError {
				details,
				source,
				error_code,
			} => format_error(
				f,
				error_code.unwrap_or(msg::error::code::server::SERVER_ERROR),
				"Server implementation error",
				details,
				source.as_deref(),
			),
			Error::LobbyError {
				details,
				source,
				error_code,
			} => format_error(
				f,
				error_code.unwrap_or(msg::error::code::lobby::LOBBY_ERROR),
				"Lobby error",
				details,
				source.as_deref(),
			),
			Error::GameError {
				details,
				source,
				error_code,
			} => format_error(
				f,
				error_code.unwrap_or(msg::error::code::game::GAME_ERROR),
				"Game error",
				details,
				source.as_deref(),
			),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match &self {
			Error::ProtocolError { .. } => None,
			Error::ServerError {
				details: _,
				source,
				error_code: _,
			} => source.as_deref(),
			Error::ImplError {
				details: _,
				source,
				error_code: _,
			} => source.as_deref(),
			Error::LobbyError {
				details: _,
				source,
				error_code: _,
			} => source.as_deref(),
			Error::GameError {
				details: _,
				source,
				error_code: _,
			} => source.as_deref(),
		}
	}
}

fn format_error(
	f: &mut Formatter<'_>,
	error_code: u32,
	error_type_msg: &str,
	details: &str,
	source: Option<&(dyn std::error::Error + 'static)>,
) -> std::fmt::Result {
	write!(
		f,
		"{} {}: {}{}",
		error_type_msg,
		error_code,
		details,
		format_error_source(source)
	)
}

fn format_error_source(e: Option<&(dyn std::error::Error + 'static)>) -> String {
	match e {
		Some(e) => format!("\n  caused by: {}", e) + format_error_source(e.source()).as_str(),
		None => "".to_owned(),
	}
}

pub(crate) use unimpl;
