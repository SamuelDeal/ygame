use crate::app::Application;
use crate::conn::handler::MsgHandler;
use crate::conn::handler::NextHandler;
use crate::conn::socket::Socket;
use crate::ep::EpRef;
use crate::err::Error;
use crate::err::JsException;
use crate::utils::console;
use crate::Result;
use ygame_protocol::v1 as msg;

use js_sys::ArrayBuffer;

#[derive(Debug)]
pub struct RunningHandler {
	request_index: i32,
}

impl RunningHandler {
	pub fn new() -> Self {
		RunningHandler { request_index: 1 }
	}

	fn on_lobby_message(&self, lobby_msg: msg::LobbyServerMessages, app: &mut Application) {
		match lobby_msg {
			msg::LobbyServerMessages::GameList { list } => {
				app.on_lobby_game_list(list.iter().map(|v| v.into()).collect())
			}
			msg::LobbyServerMessages::NewGame(info) => app.on_lobby_new_game(info.into()),
			msg::LobbyServerMessages::GameInfoChanged(info) => app.on_lobby_game_changed(info.into()),
			msg::LobbyServerMessages::GameRemoved { id } => app.on_lobby_game_removed(id),
			msg::LobbyServerMessages::GameCreated {
				request_uid,
				info,
				role,
			} => app.on_game_created(info.into(), role.into()),
			msg::LobbyServerMessages::GameJoined { info, role, moves } => {
				app.on_game_joined(info.into(), role.into(), moves.iter().map(|v| v.into()).collect())
			}
		};
	}

	fn on_game_message(&self, game_msg: msg::GameAction, ep: EpRef) {}
}

impl MsgHandler for RunningHandler {
	fn on_start(&self, ws: &Socket, _app: &mut Application) -> Result<()> {
		let msg = msg::RunningClientMessage::Lobby(msg::LobbyClientMessage::AskGameList);
		let msg = match rmp_serde::encode::to_vec(&msg) {
			Ok(msg) => msg,
			Err(e) => {
				return Err(Error::ImplError {
					details: "Serialisation failed".to_owned(),
					source: Some(Box::new(e)),
				});
			}
		};
		let msg_ref: &[u8] = &msg;
		match ws.send_buffer(msg_ref) {
			Err(e) => Err(Error::ConnectionError {
				details: "Unable to send message to the server".to_owned(),
				source: Some(Box::new(JsException::from(e))),
			}),
			Ok(_) => Ok(()),
		}
	}

	fn on_array_buf(&self, _ws: &Socket, app: &mut Application, buf: &ArrayBuffer) -> Result<NextHandler> {
		console::log!("running received: {:?}", buf);
		let array = js_sys::Uint8Array::new(buf);
		let resp: std::result::Result<msg::RunningServerMessage, rmp_serde::decode::Error> =
			rmp_serde::from_read_ref(&array.to_vec());
		match resp {
			Ok(message) => {
				match message {
					msg::RunningServerMessage::Lobby(lobby_msg) => self.on_lobby_message(lobby_msg, app),
					msg::RunningServerMessage::Game { game_id, message } => match message {
						msg::GameServerMessage::Action(action) => (),
						msg::GameServerMessage::GameActionResponse { request_id, response } => (),
						msg::GameServerMessage::UserJoin {
							user_uid,
							username,
							role,
						} => (),
						msg::GameServerMessage::UserQuit { user_uid, role } => (),
					}, //) => self.on_game_message(game_msg, app),
				};
				Ok(None)
			}
			Err(e) => Err(Error::ProtocolError {
				details: "Desserialisation error".to_owned(),
				should_reload: false,
				source: Some(Box::new(e)),
			}),
		}
	}

	fn create_game(&mut self, ws: &Socket) -> Result<()> {
		let request_id = self.request_index;
		self.request_index += 1;
		let msg = msg::RunningClientMessage::Lobby(msg::LobbyClientMessage::CreateGame {
			request_uid: request_id.to_string(),
		});
		let msg = match rmp_serde::encode::to_vec(&msg) {
			Ok(msg) => msg,
			Err(e) => {
				return Err(Error::ImplError {
					details: "Serialisation failed".to_owned(),
					source: Some(Box::new(e)),
				});
			}
		};
		let msg_ref: &[u8] = &msg;
		match ws.send_buffer(msg_ref) {
			Err(e) => Err(Error::ConnectionError {
				details: "Unable to send message to the server".to_owned(),
				source: Some(Box::new(JsException::from(e))),
			}),
			Ok(_) => Ok(()),
		}
	}

	fn join_game(&self, ws: &Socket, game_id: &str) -> Result<()> {
		panic!("Network is not ready to join game");
	}
}
