pub mod login;
pub mod running;

use crate::app::game;
use crate::app::lobby;
use ygame_protocol::v1 as msg;

impl From<msg::GameOverview> for lobby::GameOverview {
	fn from(v: msg::GameOverview) -> lobby::GameOverview {
		lobby::GameOverview {
			id: v.id,
			name: v.name,
			status: v.status.into(),
		}
	}
}

impl From<&msg::GameOverview> for lobby::GameOverview {
	fn from(v: &msg::GameOverview) -> lobby::GameOverview {
		let status = &v.status;
		lobby::GameOverview {
			id: v.id.clone(),
			name: v.name.clone(),
			status: status.into(),
		}
	}
}

impl From<msg::GameStatus> for lobby::GameStatus {
	fn from(v: msg::GameStatus) -> lobby::GameStatus {
		match v {
			msg::GameStatus::Joinable => lobby::GameStatus::Joinable,
			msg::GameStatus::Rejoinable => lobby::GameStatus::Rejoinable,
			msg::GameStatus::Full => lobby::GameStatus::Full,
			msg::GameStatus::Finished => lobby::GameStatus::Finished,
		}
	}
}

impl From<&msg::GameStatus> for lobby::GameStatus {
	fn from(v: &msg::GameStatus) -> lobby::GameStatus {
		match v {
			msg::GameStatus::Joinable => lobby::GameStatus::Joinable,
			msg::GameStatus::Rejoinable => lobby::GameStatus::Rejoinable,
			msg::GameStatus::Full => lobby::GameStatus::Full,
			msg::GameStatus::Finished => lobby::GameStatus::Finished,
		}
	}
}

impl From<msg::GameDetails> for game::GameDetails {
	fn from(v: msg::GameDetails) -> game::GameDetails {
		game::GameDetails {
			id: v.id,
			name: v.name,
			is_finished: v.is_finished,
			seat_1_username: v.seat_1_username,
			seat_2_username: v.seat_2_username,
		}
	}
}

impl From<&msg::GameDetails> for game::GameDetails {
	fn from(v: &msg::GameDetails) -> game::GameDetails {
		game::GameDetails {
			id: v.id.clone(),
			name: v.name.clone(),
			is_finished: v.is_finished,
			seat_1_username: v.seat_1_username.clone(),
			seat_2_username: v.seat_2_username.clone(),
		}
	}
}

impl From<msg::UserRole> for game::UserRole {
	fn from(v: msg::UserRole) -> game::UserRole {
		match v {
			msg::UserRole::Seat1 => game::UserRole::Seat1,
			msg::UserRole::Seat2 => game::UserRole::Seat2,
			msg::UserRole::Observer => game::UserRole::Observer,
		}
	}
}

impl From<msg::GameAction> for game::GameAction {
	fn from(v: msg::GameAction) -> game::GameAction {
		match v {
			msg::GameAction::Init => game::GameAction::Init,
			msg::GameAction::Move => game::GameAction::Move,
			msg::GameAction::Finished => game::GameAction::Finished,
		}
	}
}

impl From<&msg::GameAction> for game::GameAction {
	fn from(v: &msg::GameAction) -> game::GameAction {
		match v {
			msg::GameAction::Init => game::GameAction::Init,
			msg::GameAction::Move => game::GameAction::Move,
			msg::GameAction::Finished => game::GameAction::Finished,
		}
	}
}
