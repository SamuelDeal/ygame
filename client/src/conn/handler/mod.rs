pub mod handshake;
pub mod v1;

use crate::app::Application;
use crate::conn::socket::Socket;
use crate::err::Error;
use crate::Result;

pub type NextHandler = Option<Box<dyn MsgHandler>>;

pub trait MsgHandler: std::fmt::Debug {
	fn on_text(&self, _socket: &Socket, _app: &mut Application, _txt: &str) -> Result<NextHandler> {
		Err(Error::ProtocolError {
			details: "Unexpected text message".to_owned(),
			should_reload: false,
			source: None,
		})
	}

	fn on_array_buf(
		&self,
		_socket: &Socket,
		_app: &mut Application,
		_buf: &js_sys::ArrayBuffer,
	) -> Result<NextHandler> {
		Err(Error::ProtocolError {
			details: "Unexpected array buffer message".to_owned(),
			should_reload: false,
			source: None,
		})
	}

	fn on_blob(&self, _socket: &Socket, _app: &mut Application, _blob: &web_sys::Blob) -> Result<NextHandler> {
		Err(Error::ProtocolError {
			details: "Unexpected blob message".to_owned(),
			should_reload: false,
			source: None,
		})
	}

	fn on_start(&self, _socket: &Socket, _app: &mut Application) -> Result<()> {
		Ok(())
	}

	fn create_game(&mut self, _ws: &Socket) -> Result<()> {
		panic!("Network is not ready to create game");
	}

	fn join_game(&self, _ws: &Socket, _game_id: &str) -> Result<()> {
		panic!("Network is not ready to join game");
	}
}
