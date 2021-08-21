use serde::{Deserialize, Serialize};
use ygame_game as rules;

pub const VERSION: u32 = 1;

// ---------------- Common data ----------------------

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum GameAction {
	Init,
	Move,
	Finished,
}

// ---------------- Client messages ----------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginMessage {
	pub name: String,
	pub uid: Option<String>,
	pub session_uid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RunningClientMessage {
	Lobby(LobbyClientMessage),
	Game {
		game_id: String,
		request_id: String,
		action: GameAction,
	},
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyClientMessage {
	AskGameList,
	CreateGame { request_uid: String },
	JoinGame { game_uid: String },
}

// ---------------- Server messages ----------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponseMessage {
	pub name: String,
	pub user_uid: String,
	pub session_uid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RunningServerMessage {
	Lobby(LobbyServerMessages),
	Game {
		game_id: String,
		message: GameServerMessage,
	},
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameServerMessage {
	Action(GameAction),
	GameActionResponse {
		request_id: String,
		response: GameActionResponse,
	},
	UserJoin {
		user_uid: String,
		username: String,
		role: UserRole,
	},
	UserQuit {
		user_uid: String,
		role: UserRole,
	},
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyServerMessages {
	GameList {
		list: Vec<GameOverview>,
	},
	GameCreated {
		request_uid: String,
		info: GameDetails,
		role: UserRole,
	},
	NewGame(GameOverview),
	GameInfoChanged(GameOverview),
	GameJoined {
		info: GameDetails,
		role: UserRole,
		moves: Vec<GameAction>,
	},
	GameRemoved {
		id: String,
	},
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum UserRole {
	Seat1,
	Seat2,
	Observer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameOverview {
	pub id: String,
	pub name: String,
	pub status: GameStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameDetails {
	pub id: String,
	pub name: String,
	pub is_finished: bool,
	pub seat_1_username: Option<String>,
	pub seat_2_username: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameStatus {
	Joinable = 1,
	Rejoinable = 2,
	Full = 3,
	Finished = 4,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameActionResponse {
	Illegal { reason: u32 },
	Ok,
}

// ------------------------ Some rules enum casting routines ---------------------------------

impl From<rules::UserRole> for UserRole {
	fn from(val: rules::UserRole) -> Self {
		match val {
			rules::UserRole::Observer => UserRole::Observer,
			rules::UserRole::Seat1 => UserRole::Seat1,
			rules::UserRole::Seat2 => UserRole::Seat2,
		}
	}
}

impl From<&rules::UserRole> for UserRole {
	fn from(val: &rules::UserRole) -> Self {
		match val {
			rules::UserRole::Observer => UserRole::Observer,
			rules::UserRole::Seat1 => UserRole::Seat1,
			rules::UserRole::Seat2 => UserRole::Seat2,
		}
	}
}

impl From<rules::Action> for GameAction {
	fn from(val: rules::Action) -> Self {
		match val {
			rules::Action::Init => GameAction::Init,
			rules::Action::Move => GameAction::Move,
			rules::Action::Finished => GameAction::Finished,
		}
	}
}

impl From<&rules::Action> for GameAction {
	fn from(val: &rules::Action) -> Self {
		match val {
			rules::Action::Init => GameAction::Init,
			rules::Action::Move => GameAction::Move,
			rules::Action::Finished => GameAction::Finished,
		}
	}
}

impl From<&GameAction> for rules::Action {
	fn from(val: &GameAction) -> Self {
		match val {
			GameAction::Init => rules::Action::Init,
			GameAction::Move => rules::Action::Move,
			GameAction::Finished => rules::Action::Finished,
		}
	}
}

impl From<GameAction> for rules::Action {
	fn from(val: GameAction) -> Self {
		match val {
			GameAction::Init => rules::Action::Init,
			GameAction::Move => rules::Action::Move,
			GameAction::Finished => rules::Action::Finished,
		}
	}
}
