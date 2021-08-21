use crate::actor_msg as actmsg;
use crate::client::Client;
use crate::client::ClientContext;
use crate::client::ClientStatus;
use crate::client::MessageForwarder;
use crate::client::Protocol;
use crate::error::*;
use crate::game;
use crate::user;
use crate::{Result, StdResult};

use actix::prelude::*;
use actix_web::web::Bytes;
use ygame_game as rules;
use ygame_protocol::error::code as error_code;
use ygame_protocol::v1 as msg;

use uuid::Uuid;

impl MessageForwarder<actmsg::GameMessage, 1> for Client {
	fn forward_msg(&self, ctx: &mut ClientContext, msg: &actmsg::GameMessage) -> Result<()> {
		let response: msg::RunningServerMessage = msg.clone().into();
		self.v1_send_response(ctx, &response)
	}
}

impl MessageForwarder<actmsg::LobbyMessage, 1> for Client {
	fn forward_msg(&self, ctx: &mut ClientContext, msg: &actmsg::LobbyMessage) -> Result<()> {
		let user = self.required_login()?;
		let response = match msg {
			actmsg::LobbyMessage::NewGame(game_info) => msg::RunningServerMessage::Lobby(
				msg::LobbyServerMessages::NewGame(map_v1_game_overview(user, &game_info)),
			),
			actmsg::LobbyMessage::GameStatusChanged(game_info) => msg::RunningServerMessage::Lobby(
				msg::LobbyServerMessages::GameInfoChanged(map_v1_game_overview(user, &game_info)),
			),
			actmsg::LobbyMessage::GameRemoved { game_id } => {
				msg::RunningServerMessage::Lobby(msg::LobbyServerMessages::GameRemoved {
					id: game_id.to_string(),
				})
			}
		};
		self.v1_send_response(ctx, &response)
	}
}

impl MessageForwarder<actmsg::GameList, 1> for Client {
	fn forward_msg(&self, ctx: &mut ClientContext, msg: &actmsg::GameList) -> Result<()> {
		let user = self.required_login()?;
		let response = msg::RunningServerMessage::Lobby(msg::LobbyServerMessages::GameList {
			list: msg.list.iter().map(|k| map_v1_game_overview(user, k)).collect(),
		});
		self.v1_send_response(ctx, &response)
	}
}

impl From<actmsg::GameMessage> for msg::RunningServerMessage {
	fn from(msg: actmsg::GameMessage) -> msg::RunningServerMessage {
		match msg.details {
			actmsg::GameMessageDetails::Action(action) => msg::RunningServerMessage::Game {
				game_id: msg.game_id.to_string(),
				message: msg::GameServerMessage::Action(action.into()),
			},
			actmsg::GameMessageDetails::UserJoin {
				user_uid,
				user_name,
				user_seat,
			} => msg::RunningServerMessage::Game {
				game_id: msg.game_id.to_string(),
				message: msg::GameServerMessage::UserJoin {
					user_uid: user_uid.to_string(),
					username: user_name,
					role: user_seat.into(),
				},
			},
			actmsg::GameMessageDetails::UserQuit { user_uid, user_seat } => msg::RunningServerMessage::Game {
				game_id: msg.game_id.to_string(),
				message: msg::GameServerMessage::UserQuit {
					user_uid: user_uid.to_string(),
					role: user_seat.into(),
				},
			},
		}
	}
}

impl From<actmsg::GameActionResponse> for msg::GameActionResponse {
	fn from(msg: actmsg::GameActionResponse) -> msg::GameActionResponse {
		match msg {
			actmsg::GameActionResponse::Illegal { reason } => msg::GameActionResponse::Illegal { reason },
			actmsg::GameActionResponse::Ok => msg::GameActionResponse::Ok,
		}
	}
}

fn map_v1_game_overview(user: &user::User, game_info: &game::Info) -> msg::GameOverview {
	msg::GameOverview {
		id: game_info.id.to_string(),
		name: game_info.name.clone(),
		status: if game_info.status == game::Status::Finished {
			msg::GameStatus::Finished
		} else if Some(user.uid) == game_info.seat_1 || Some(user.uid) == game_info.seat_2 {
			msg::GameStatus::Rejoinable
		} else if game_info.seat_1.is_none() || game_info.seat_2.is_none() {
			msg::GameStatus::Joinable
		} else {
			msg::GameStatus::Full
		},
	}
}

impl Protocol<1> for Client {
	fn send_game_joined(&self, ctx: &mut ClientContext, game_id: Uuid, res: &actmsg::GameJoined) -> Result<()> {
		let msg = msg::RunningServerMessage::Lobby(msg::LobbyServerMessages::GameJoined {
			info: msg::GameDetails {
				id: game_id.to_string(),
				name: res.game_name.clone(),
				is_finished: res.moves.contains(&rules::Action::Finished),
				seat_1_username: res.seat_1_username.clone(),
				seat_2_username: res.seat_2_username.clone(),
			},
			role: res.user_role.into(),
			moves: res.moves.iter().map(|v| v.into()).collect(),
		});
		self.v1_send_response(ctx, &msg)
	}

	fn send_game_created(&self, ctx: &mut ClientContext, res: &actmsg::GameCreated, request_uid: String) -> Result<()> {
		let login = self.required_login()?;
		let msg = msg::RunningServerMessage::Lobby(msg::LobbyServerMessages::GameCreated {
			request_uid,
			info: msg::GameDetails {
				id: res.game_uid.to_string(),
				name: res.game_name.clone(),
				is_finished: false,
				seat_1_username: if res.user_seat == rules::UserRole::Seat1 {
					Some(login.name.clone())
				} else {
					None
				},
				seat_2_username: if res.user_seat == rules::UserRole::Seat1 {
					Some(login.name.clone())
				} else {
					None
				},
			},
			role: res.user_seat.into(),
		});
		self.v1_send_response(ctx, &msg)
	}

	fn send_game_action_response(
		&self,
		ctx: &mut ClientContext,
		request_id: String,
		game_id: Uuid,
		msg: actmsg::GameActionResponse,
	) -> Result<()> {
		let msg = msg::RunningServerMessage::Game {
			game_id: game_id.to_string(),
			message: msg::GameServerMessage::GameActionResponse {
				request_id,
				response: msg.into(),
			},
		};
		self.v1_send_response(ctx, &msg)
	}

	fn on_txt_message(&mut self, _: String, _: &mut ClientContext) -> Result<()> {
		Err(Error::ProtocolError {
			details: "Unexpected text message".to_owned(),
			source: None,
			error_code: Some(error_code::protocol::UNEXPECTED_STRING),
		})
	}

	fn on_bin_message(&mut self, msg: Bytes, ctx: &mut ClientContext) -> Result<()> {
		match self.status {
			ClientStatus::Handshake => Err(Error::ProtocolError {
				details: "Unexpected bin message".to_owned(),
				source: None,
				error_code: Some(error_code::protocol::UNEXPECTED_BIN),
			}),
			ClientStatus::Login => self.v1_on_login(ctx, msg),
			ClientStatus::Running => self.v1_on_running(ctx, msg),
		}
	}
}

trait ProtocolImplv1 {
	fn v1_send_response(&self, ctx: &mut ClientContext, msg: &msg::RunningServerMessage) -> Result<()>;

	fn v1_on_login(&mut self, ctx: &mut ClientContext, bin: Bytes) -> Result<()>;
	fn v1_on_running(&mut self, ctx: &mut ClientContext, bin: Bytes) -> Result<()>;

	fn v1_parse_game_id(&self, game_id: String) -> Result<Uuid>;
	fn v1_on_client_lobby(&mut self, ctx: &mut ClientContext, msg: msg::LobbyClientMessage) -> Result<()>;
}

impl ProtocolImplv1 for Client {
	fn v1_on_login(&mut self, ctx: &mut ClientContext, bin: Bytes) -> Result<()> {
		let bin: &[u8] = bin.as_ref();
		let msg: StdResult<msg::LoginMessage, rmp_serde::decode::Error> = rmp_serde::from_read(bin);
		match msg {
			Ok(msg) => {
				self.lobby_addr
					.send(actmsg::RegisterUserMessage {
						name: msg.name,
						session_uid: msg.session_uid.map(|s| Uuid::parse_str(s.as_str()).ok()).flatten(),
						user_uid: msg.uid.map(|s| Uuid::parse_str(s.as_str()).ok()).flatten(),
						client_id: self.id,
					})
					.into_actor(self)
					.then(|res, this, ctx| {
						let res = match res {
							Ok(res) => {
								this.user_info = Some(user::User {
									name: res.name.clone(),
									session_uid: res.session_uid.clone(),
									uid: res.user_uid,
								});
								let response = msg::LoginResponseMessage {
									name: res.name,
									user_uid: res.user_uid.to_string(),
									session_uid: res.session_uid.to_string(),
								};
								this.status = ClientStatus::Running;
								match rmp_serde::encode::to_vec(&response) {
									Ok(response) => ctx.binary(response),
									Err(e) => this.on_serialize_error(ctx, e),
								}
								Ok(())
							}
							Err(e) => Err(Client::map_mailbox_err(&e, "lobby")),
						};
						this.send_error_if_failed(ctx, res);
						fut::ready(())
					})
					.wait(ctx);
				Ok(())
			}
			Err(err) => Err(Error::ProtocolError {
				details: "Bad Login".to_owned(),
				source: Some(Box::new(err)),
				error_code: Some(error_code::protocol::BAD_HANDSHAKE),
			}),
		}
	}

	fn v1_on_running(&mut self, ctx: &mut ClientContext, bin: Bytes) -> Result<()> {
		let bin: &[u8] = bin.as_ref();
		let msg: StdResult<msg::RunningClientMessage, rmp_serde::decode::Error> = rmp_serde::from_read(bin);
		match msg {
			Ok(msg) => match msg {
				msg::RunningClientMessage::Lobby(msg) => self.v1_on_client_lobby(ctx, msg),
				msg::RunningClientMessage::Game {
					game_id,
					request_id,
					action,
				} => self.on_game_action(ctx, request_id, self.v1_parse_game_id(game_id)?, action.into()),
			},
			Err(err) => Err(Error::ProtocolError {
				details: "Bad message".to_owned(),
				source: Some(Box::new(err)),
				error_code: Some(error_code::protocol::INVALID_MESSAGE),
			}),
		}
	}

	fn v1_send_response(&self, ctx: &mut ClientContext, msg: &msg::RunningServerMessage) -> Result<()> {
		match rmp_serde::encode::to_vec(&msg) {
			Ok(response) => {
				ctx.binary(response);
				Ok(())
			}
			Err(e) => Err(Error::ServerError {
				details: "Serialization issue".to_owned(),
				source: Some(Box::new(e)),
				error_code: Some(error_code::server::SERIALIZATION_ERROR),
			}),
		}
	}

	fn v1_on_client_lobby(&mut self, ctx: &mut ClientContext, msg: msg::LobbyClientMessage) -> Result<()> {
		match msg {
			msg::LobbyClientMessage::AskGameList => self.on_ask_game_list(ctx),
			msg::LobbyClientMessage::CreateGame { request_uid } => self.on_create_game(ctx, request_uid),
			msg::LobbyClientMessage::JoinGame { game_uid } => self.on_join_game(ctx, self.v1_parse_game_id(game_uid)?),
		}
	}

	fn v1_parse_game_id(&self, game_id: String) -> Result<Uuid> {
		match Uuid::parse_str(game_id.as_str()) {
			Ok(v) => Ok(v),
			Err(e) => Err(Error::ProtocolError {
				details: "Invalid game id".to_owned(),
				source: Some(Box::new(e)),
				error_code: Some(error_code::protocol::INVALID_GAME_ID),
			}),
		}
	}
}
