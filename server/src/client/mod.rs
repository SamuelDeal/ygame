mod v1;

use std::collections::BTreeMap;
use std::result::Result as StdResult;
use std::time::Duration;
use std::time::Instant;

use crate::actor_msg as actmsg;
use crate::actor_msg::GameMessage;
use crate::error::*;
use crate::game;
use crate::lobby::Lobby;
use crate::log;
use crate::user;
use crate::Result;
use ygame_game as rules;

use actix::fut::ActorFuture;
use actix::fut::Map;
use actix::prelude::*;
use actix_web::web::Bytes;
use actix_web_actors::ws;
use msg::error::code::game as game_error;
use msg::error::code::lobby as lobby_error;
use msg::error::code::protocol as protocol_error;
use msg::error::code::server as server_error;
use msg::v1::LobbyClientMessage;
use uuid::Uuid;
use ygame_protocol as msg;

#[derive(PartialEq)]
pub enum ClientStatus {
	Handshake,
	Login,
	Running,
}

/// Define HTTP actor
pub struct Client {
	id: Uuid,
	status: ClientStatus,
	lobby_addr: Addr<Lobby>,
	games: BTreeMap<Uuid, Recipient<actmsg::GameAction>>,
	protocol_version: Option<u32>,
	user_info: Option<user::User>,
	heart_beat: Instant,
}

type ClientContext = <Client as Actor>::Context;

impl Actor for Client {
	type Context = ws::WebsocketContext<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {
		self.beat(ctx);

		let addr = ctx.address();
		self.lobby_addr
			.send(actmsg::Connect {
				client_addr: addr.recipient(),
				client_id: self.id,
			})
			.into_actor(self)
			.then(|res, _, ctx| {
				match res {
					Ok(_res) => (),
					_ => ctx.stop(),
				}
				fut::ready(())
			})
			.wait(ctx);
	}

	fn stopping(&mut self, _: &mut Self::Context) -> Running {
		log::info!("Disconnect client from stopping");
		self.lobby_addr.do_send(actmsg::Disconnect { client_id: self.id });
		Running::Stop
	}
}

impl Client {
	pub fn new(lobby_addr: Addr<Lobby>) -> Client {
		Client {
			id: Uuid::new_v4(),
			status: ClientStatus::Handshake,
			protocol_version: None,
			user_info: None,
			lobby_addr,
			games: BTreeMap::new(),
			heart_beat: Instant::now(),
		}
	}

	fn beat(&self, ctx: &mut ws::WebsocketContext<Self>) {
		ctx.run_interval(Duration::from_secs(5), |act, ctx| {
			if Instant::now().duration_since(act.heart_beat) > Duration::from_secs(10) {
				log::info!("Disconnecting failed heartbeat");
				act.lobby_addr.do_send(actmsg::Disconnect { client_id: act.id });
				ctx.stop();
				return;
			}
			ctx.ping(b"ping");
		});
	}

	fn choose_protocol(proposed_protocols: &mut Vec<u32>) -> Option<u32> {
		proposed_protocols.sort_unstable();
		proposed_protocols.reverse();
		let sorted_protocols: &Vec<u32> = proposed_protocols;
		for &protocol_id in sorted_protocols {
			if protocol_id == msg::v1::VERSION {
				return Some(protocol_id);
			}
		}
		None
	}

	fn required_login(&self) -> Result<&user::User> {
		match &self.user_info {
			Some(user_info) => Ok(user_info),
			None => Err(Error::ProtocolError {
				details: "Message is refused as long as you are not logged in".to_owned(),
				error_code: Some(msg::error::code::protocol::NEED_LOGIN),
				source: None,
			}),
		}
	}

	fn required_protocol(&self) -> Result<u32> {
		match self.protocol_version {
			Some(protocol_version) => Ok(protocol_version),
			None => Err(Error::ProtocolError {
				details: "You need to run the handshake process".to_owned(),
				error_code: Some(msg::error::code::protocol::NEED_HANDSHAKE),
				source: None,
			}),
		}
	}

	fn on_txt_message(&mut self, msg: String, ctx: &mut ClientContext) -> Result<()> {
		match self.status {
			ClientStatus::Handshake => self.on_handshake(ctx, msg),
			_ => match self.required_protocol()? {
				1 => Protocol::<1>::on_txt_message(self, msg, ctx),
				protocol => Err(unimpl!("on_txt for protocol {}", protocol)),
			},
		}
	}

	fn on_bin_message(&mut self, msg: Bytes, ctx: &mut ClientContext) -> Result<()> {
		match self.status {
			ClientStatus::Handshake => Err(Error::ProtocolError {
				details: "Unexpected bin message".to_owned(),
				source: None,
				error_code: Some(protocol_error::UNEXPECTED_BIN),
			}),
			_ => match self.required_protocol()? {
				1 => Protocol::<1>::on_bin_message(self, msg, ctx),
				protocol => Err(unimpl!("on_bin for protocol {}", protocol)),
			},
		}
	}

	fn send_error_if_failed<T>(&mut self, ctx: &mut ClientContext, maybe_err: Result<T>) {
		if let Err(e) = maybe_err {
			self.send_error(ctx, &e);
		}
	}

	fn on_serialize_error(&mut self, ctx: &mut ClientContext, error: rmp_serde::encode::Error) {
		self.send_error(
			ctx,
			&Error::ServerError {
				details: "Serialization issue".to_owned(),
				source: Some(Box::new(error)),
				error_code: Some(server_error::SERIALIZATION_ERROR),
			},
		)
	}

	fn map_mailbox_err(e: &actix::MailboxError, actor_name: &str) -> Error {
		Error::ServerError {
			details: format!("Unable to contact {} actor", actor_name),
			source: Some(Box::new(e.clone())),
			error_code: Some(msg::error::code::server::MAILBOX_ERROR),
		}
	}

	fn send_error(&mut self, ctx: &mut ClientContext, error: &Error) {
		match error {
			Error::ProtocolError { .. } | Error::GameError { .. } => log::warning!("{}", error),
			_ => log::error!("{}", error),
		}
		if let Error::ProtocolError { .. } = error {
			self.games = BTreeMap::new();
			self.protocol_version = None;
			self.status = ClientStatus::Handshake;
		}

		let resp = match error {
			Error::ProtocolError {
				details, error_code, ..
			} => msg::error::ErrorMessage {
				error_code: Some(error_code.unwrap_or(msg::error::code::protocol::PROTOCOL_ERROR)),
				error_description: details.clone(),
				should_handshake: true,
				..Default::default()
			},
			Error::ImplError {
				details, error_code, ..
			} => msg::error::ErrorMessage {
				error_code: Some(error_code.unwrap_or(msg::error::code::server::SERVER_ERROR)),
				error_description: details.clone(),
				..Default::default()
			},
			Error::ServerError {
				details, error_code, ..
			} => msg::error::ErrorMessage {
				error_code: Some(error_code.unwrap_or(msg::error::code::server::SERVER_ERROR)),
				error_description: details.clone(),
				..Default::default()
			},
			Error::GameError {
				details, error_code, ..
			} => msg::error::ErrorMessage {
				error_code: Some(error_code.unwrap_or(msg::error::code::game::GAME_ERROR)),
				error_description: details.clone(),
				..Default::default()
			},
			Error::LobbyError {
				details, error_code, ..
			} => msg::error::ErrorMessage {
				error_code: Some(error_code.unwrap_or(msg::error::code::server::SERVER_ERROR)),
				error_description: details.clone(),
				..Default::default()
			},
		};
		match serde_json::to_string(&resp) {
			Ok(resp) => ctx.text(resp),
			Err(error) => log::log_err(&error),
		}
	}

	fn on_handshake(&mut self, ctx: &mut ClientContext, text: String) -> Result<()> {
		let hello_msg: serde_json::Result<msg::handshake::HelloMessage> = serde_json::from_str(&text);
		match hello_msg {
			Ok(mut hello_msg) => {
				if hello_msg.known_protocols.is_empty() {
					Err(Error::ProtocolError {
						details: "No known protocols".to_owned(),
						source: None,
						error_code: Some(protocol_error::NO_PROTOCOL_VERSION),
					})
				} else {
					let best_protocol_id: Option<u32> = Client::choose_protocol(&mut hello_msg.known_protocols);
					let resp = if let Some(best_protocol_id) = best_protocol_id {
						self.protocol_version = Some(best_protocol_id);
						self.status = ClientStatus::Login;
						log::debug!("Protocol choosen: {}", best_protocol_id);
						msg::handshake::HelloResponseMessage::Success {
							protocol_version: best_protocol_id,
						}
					} else {
						log::warning!("Protocol handshake failure");
						msg::handshake::HelloResponseMessage::Failure { should_reload: true }
					};
					ctx.text(serde_json::to_string(&resp).unwrap());
					Ok(())
				}
			}
			Err(err) => Err(Error::ProtocolError {
				details: "No known protocols".to_owned(),
				source: Some(Box::new(err)),
				error_code: Some(protocol_error::BAD_HANDSHAKE),
			}),
		}
	}

	fn on_ask_game_list(&self, ctx: &mut ClientContext) -> Result<()> {
		self.required_login()?;
		let protocol = self.required_protocol()?;
		self.lobby_addr
			.send(actmsg::AskGameList {})
			.into_actor(self)
			.then(move |res, this, ctx| {
				let res = match res {
					Ok(res) => this.forward_msg_with_protocol(protocol, ctx, &res),
					Err(e) => Err(Client::map_mailbox_err(&e, "lobby")),
				};
				this.send_error_if_failed(ctx, res);
				fut::ready(())
			})
			.wait(ctx);
		Ok(())
	}

	fn on_create_game(&mut self, ctx: &mut ClientContext, request_uid: String) -> Result<()> {
		let addr = ctx.address();
		let user_info = self.required_login()?;
		let protocol = self.required_protocol()?;
		self.lobby_addr
			.send(actmsg::CreateGame {
				user_uid: user_info.uid,
				user_name: user_info.name.clone(),
				client_id: self.id,
				client_addr: addr.recipient(),
			})
			.into_actor(self)
			.then(move |res, this, ctx| {
				let res = match res {
					Ok(res) => this.on_game_created(protocol, ctx, &res, request_uid),
					Err(e) => Err(Client::map_mailbox_err(&e, "lobby")),
				};
				this.send_error_if_failed(ctx, res);
				fut::ready(())
			})
			.wait(ctx);
		Ok(())
	}

	fn on_join_game(&self, ctx: &mut ClientContext, game_id: Uuid) -> Result<()> {
		let addr = ctx.address();
		let user_info = self.required_login()?;
		let protocol = self.required_protocol()?;
		let user_uid = user_info.uid.clone();
		let user_name = user_info.name.clone();
		if self.games.contains_key(&game_id) {
			return Err(Error::LobbyError {
				details: "Game already joined".to_owned(),
				source: None,
				error_code: Some(lobby_error::GAME_ALREADY_JOINED),
			});
		}
		self.lobby_addr
			.send(actmsg::GetGame {
				game_id: game_id,
				client_addr: addr.clone().recipient(),
			})
			.into_actor(self)
			.then(move |res, this, ctx| {
				let res = match res {
					Ok(actmsg::GameFound::Success { game }) => {
						game.send(actmsg::JoinGame {
							user_uid,
							user_name,
							client_id: this.id,
							client_addr: addr.recipient(),
						})
						.into_actor(this)
						.then(move |res, this, ctx| {
							let res = match res {
								Ok(res) => this.on_game_joined(protocol, ctx, game_id, res, game.clone().recipient()),
								Err(e) => Err(Client::map_mailbox_err(&e, "game")),
							};
							this.send_error_if_failed(ctx, res);
							fut::ready(())
						})
						.wait(ctx);
						Ok(())
					}
					Ok(actmsg::GameFound::Failure) => Err(Error::LobbyError {
						details: format!("Unable to find game {}", game_id),
						source: None,
						error_code: Some(lobby_error::GAME_DOESNT_EXISTS),
					}),
					Err(e) => Err(Client::map_mailbox_err(&e, "lobby")),
				};
				this.send_error_if_failed(ctx, res);
				fut::ready(())
			})
			.wait(ctx);
		Ok(())
	}

	fn on_game_joined(
		&mut self,
		protocol: u32,
		ctx: &mut ClientContext,
		game_uid: Uuid,
		msg: actmsg::GameJoined,
		game_addr: Recipient<actmsg::GameAction>,
	) -> Result<()> {
		self.games.insert(game_uid, game_addr.clone());
		match protocol {
			1 => Protocol::<1>::send_game_joined(self, ctx, game_uid, &msg),
			_ => Err(unimpl!("send_game_joined for protocol {}", protocol)),
		}
	}

	fn on_game_created(
		&mut self,
		protocol: u32,
		ctx: &mut ClientContext,
		msg: &actmsg::GameCreated,
		request_uid: String,
	) -> Result<()> {
		self.games.insert(msg.game_uid, msg.game_addr.clone());
		match protocol {
			1 => Protocol::<1>::send_game_created(self, ctx, msg, request_uid),
			_ => Err(unimpl!("on_game_created for protocol {}", protocol)),
		}
	}

	fn on_game_action(
		&mut self,
		ctx: &mut ClientContext,
		request_id: String,
		game_id: Uuid,
		msg: rules::Action,
	) -> Result<()> {
		let user_uid = self.required_login()?.uid.clone();
		let game = match self.games.get(&game_id) {
			Some(v) => Ok(v),
			None => Err(Error::LobbyError {
				details: "You should join the game first".to_owned(),
				source: None,
				error_code: Some(lobby_error::GAME_NOT_JOINED),
			}),
		}?;
		game.send(actmsg::GameAction {
			action: msg,
			user: Some(user_uid),
		})
		.into_actor(self)
		.then(move |res, this, ctx| {
			let res = match res {
				Ok(msg) => this.send_game_action_response(ctx, request_id, game_id, msg),
				Err(e) => Err(Client::map_mailbox_err(&e, "lobby")),
			};
			this.send_error_if_failed(ctx, res);
			fut::ready(())
		})
		.wait(ctx);
		Ok(())
	}

	fn forward_msg<T>(&self, ctx: &mut ClientContext, msg: &T) -> Result<()>
	where
		Self: MessageForwarder<T, 1>,
	{
		let protocol = self.required_protocol()?;
		self.forward_msg_with_protocol(protocol, ctx, msg)
	}

	fn forward_msg_with_protocol<T>(&self, protocol: u32, ctx: &mut ClientContext, msg: &T) -> Result<()>
	where
		Self: MessageForwarder<T, 1>,
	{
		match protocol {
			1 => MessageForwarder::<T, 1>::forward_msg(self, ctx, msg),
			_ => Err(unimpl!("forward_msg match for protocol {}", protocol)),
		}
	}
}

impl StreamHandler<StdResult<ws::Message, ws::ProtocolError>> for Client {
	fn handle(&mut self, msg: StdResult<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		match msg {
			Ok(ws::Message::Ping(msg)) => {
				self.heart_beat = Instant::now();
				ctx.pong(&msg)
			}
			Ok(ws::Message::Pong(_msg)) => {
				self.heart_beat = Instant::now();
			}
			Ok(ws::Message::Text(msg)) => {
				log::debug!("text: {}", msg);
				if msg == "ping" {
					self.heart_beat = Instant::now();
					ctx.text("pong")
				} else if msg == "pong" {
					self.heart_beat = Instant::now();
				} else {
					let disconnect_msg: serde_json::Result<msg::handshake::DisconnectMessage> =
						serde_json::from_str(&msg);
					if disconnect_msg.is_ok() {
						log::info!("Disconnect client from client message");
						self.lobby_addr.do_send(actmsg::Disconnect { client_id: self.id });
						ctx.stop();
					} else if let Err(err) = self.on_txt_message(msg, ctx) {
						self.send_error(ctx, &err);
					}
				}
			}
			Ok(ws::Message::Binary(bin)) => {
				if let Err(err) = self.on_bin_message(bin, ctx) {
					self.send_error(ctx, &err);
				}
			}
			Ok(ws::Message::Close(reason)) => {
				log::info!("client closed the connection with reason {:?}", reason);
			}
			_ => self.send_error(
				ctx,
				&Error::ProtocolError {
					details: format!("Unexpected message type {:?}", msg),
					source: None,
					error_code: Some(protocol_error::UNEXPECTED_OTHER),
				},
			),
		}
	}
}

impl Handler<actmsg::LobbyMessage> for Client {
	type Result = ();

	fn handle(&mut self, msg: actmsg::LobbyMessage, ctx: &mut Self::Context) {
		if let actmsg::LobbyMessage::GameRemoved { game_id } = msg {
			self.games.remove(&game_id);
		}
		let res = self.forward_msg(ctx, &msg);
		self.send_error_if_failed(ctx, res);
	}
}

impl Handler<actmsg::GameMessage> for Client {
	type Result = ();

	fn handle(&mut self, msg: actmsg::GameMessage, ctx: &mut Self::Context) {
		let res = self.forward_msg(ctx, &msg);
		self.send_error_if_failed(ctx, res);
	}
}

trait MessageForwarder<T, const PROTOCOL: u32> {
	fn forward_msg(&self, _ctx: &mut ClientContext, _msg: &T) -> Result<()> {
		Err(unimpl!("MsgForwarder for {} and protocol {}", stringify!(T), PROTOCOL))
	}

	fn protocol() -> u32 {
		PROTOCOL
	}
}

trait Protocol<const PROTOCOL: u32> {
	fn on_txt_message(&mut self, msg: String, ctx: &mut ClientContext) -> Result<()>;
	fn on_bin_message(&mut self, msg: Bytes, ctx: &mut ClientContext) -> Result<()>;

	fn send_game_joined(&self, ctx: &mut ClientContext, game_id: Uuid, res: &actmsg::GameJoined) -> Result<()>;
	fn send_game_created(&self, ctx: &mut ClientContext, res: &actmsg::GameCreated, request_uid: String) -> Result<()>;
	fn send_game_action_response(
		&self,
		ctx: &mut ClientContext,
		request_id: String,
		game_id: Uuid,
		msg: actmsg::GameActionResponse,
	) -> Result<()>;

	fn protocol() -> u32 {
		PROTOCOL
	}
}
