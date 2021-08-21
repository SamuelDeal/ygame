use actix::prelude::{Message, MessageResponse, Recipient};
use actix::Addr;
use uuid::Uuid;

use crate::game;
use ygame_game as rules;
use ygame_protocol as msg;

use std::result::Result as StdResult;

#[derive(MessageResponse, Clone, Debug)]
pub struct RegisterUserResponse {
	pub name: String,
	pub session_uid: Uuid,
	pub user_uid: Uuid,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "RegisterUserResponse")]
pub struct RegisterUserMessage {
	pub name: String,
	pub session_uid: Option<Uuid>,
	pub user_uid: Option<Uuid>,
	pub client_id: Uuid,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct Connect {
	pub client_id: Uuid,
	pub client_addr: Recipient<LobbyMessage>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct Disconnect {
	pub client_id: Uuid,
}

#[derive(MessageResponse, Clone, Debug)]
pub struct GameList {
	pub list: Vec<game::Info>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "GameList")]
pub struct AskGameList {}

#[derive(Message, Clone, Debug)]
#[rtype(result = "GameCreated")]
pub struct CreateGame {
	pub user_uid: Uuid,
	pub user_name: String,
	pub client_id: Uuid,
	pub client_addr: Recipient<GameMessage>,
}

#[derive(MessageResponse, Clone, Debug)]
pub struct GameCreated {
	pub game_uid: Uuid,
	pub game_name: String,
	pub game_addr: Recipient<GameAction>,
	pub user_seat: rules::UserRole,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "GameFound")]
pub struct GetGame {
	pub game_id: Uuid,
	pub client_addr: Recipient<GameMessage>,
}

#[derive(MessageResponse, Clone, Debug)]
pub enum GameFound {
	Success { game: Addr<game::Game> },
	Failure,
}

#[derive(MessageResponse, Clone, Debug)]
pub struct GameJoined {
	pub game_name: String,
	pub user_role: rules::UserRole,
	pub seat_1_username: Option<String>,
	pub seat_2_username: Option<String>,
	pub moves: Vec<rules::Action>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "GameJoined")]
pub struct JoinGame {
	//pub game_id: Uuid,
	pub user_uid: Uuid,
	pub user_name: String,
	pub client_id: Uuid,
	pub client_addr: Recipient<GameMessage>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub enum LobbyMessage {
	NewGame(game::Info),
	GameStatusChanged(game::Info),
	GameRemoved { game_id: Uuid },
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct GameClosed {
	pub game_id: Uuid,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct GameMessage {
	pub game_id: Uuid,
	pub details: GameMessageDetails,
}

#[derive(Clone, Debug)]
pub enum GameMessageDetails {
	Action(rules::Action),
	UserJoin {
		user_uid: Uuid,
		user_name: String,
		user_seat: rules::UserRole,
	},
	UserQuit {
		user_uid: Uuid,
		user_seat: rules::UserRole,
	},
}

#[derive(MessageResponse, Clone, Debug)]
pub enum GameActionResponse {
	Illegal { reason: u32 },
	Ok,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "GameActionResponse")]
pub struct GameAction {
	pub action: rules::Action,
	pub user: Option<Uuid>,
}

#[derive(MessageResponse, Debug)]
pub struct ResultMsg(pub StdResult<(), String>);
