use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ErrorMessage {
	pub error_code: Option<u32>,
	pub error_description: String,
	pub should_reload: bool,
	pub should_reconnect: bool,
	pub should_handshake: bool,
}

pub mod code {
	pub mod protocol {
		pub const PROTOCOL_ERROR: u32 = 100;
		pub const NO_PROTOCOL_VERSION: u32 = 101;
		pub const UNEXPECTED_STRING: u32 = 102;
		pub const UNEXPECTED_BIN: u32 = 103;
		pub const UNEXPECTED_OTHER: u32 = 104;
		pub const BAD_HANDSHAKE: u32 = 105;
		pub const INVALID_MESSAGE: u32 = 106;
		pub const NEED_LOGIN: u32 = 107;
		pub const NEED_HANDSHAKE: u32 = 108;
		pub const INVALID_GAME_ID: u32 = 109;
	}
	pub mod server {
		pub const SERVER_ERROR: u32 = 200;
		pub const UNIMPL_ERROR: u32 = 201;
		pub const SERIALIZATION_ERROR: u32 = 202;
		pub const MAILBOX_ERROR: u32 = 203;
	}
	pub mod lobby {
		pub const LOBBY_ERROR: u32 = 300;
		pub const GAME_ALREADY_JOINED: u32 = 301;
		pub const GAME_DOESNT_EXISTS: u32 = 302;
		pub const GAME_NOT_JOINED: u32 = 303;
	}
	pub mod game {
		pub const GAME_ERROR: u32 = 400;
		pub const ILLEGAL_MOVE: u32 = 401;
		pub const NOT_YOUR_TURN: u32 = 402;
	}
}

impl Default for ErrorMessage {
	fn default() -> ErrorMessage {
		ErrorMessage {
			error_code: Some(code::server::SERVER_ERROR),
			error_description: "Unknown server error".to_owned(),
			should_reload: false,
			should_reconnect: false,
			should_handshake: false,
		}
	}
}
