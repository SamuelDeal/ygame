use crate::utils::history;

#[derive(Clone, Debug, PartialEq)]
pub enum AppState {
	Loading,
	Login(LoginState),
	ConnectingToLobby,
	Lobby,
	ConnectingToGame { game_uid: String },
	InNetGame { game_uid: String },
	InLocalGame,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoginState {
	NewUser,
	ExistingUser,
	ChooseGameMode,
	OpponentName,
}

impl AppState {
	pub fn from_url(url: String, subfolder: &Option<String>) -> Option<AppState> {
		if !url.starts_with("/") {
			return None;
		}
		let mut url = &url[1..];
		if let Some(subfolder) = subfolder {
			if !url.starts_with(format!("{}/", subfolder).as_str()) {
				return None;
			}
			url = &url[1 + subfolder.len()..];
		}
		if url == "lobby" {
			Some(AppState::Lobby)
		} else if url == "local" {
			Some(AppState::InLocalGame)
		} else if url.starts_with("game/") && url.len() > 5 {
			Some(AppState::InNetGame {
				game_uid: url[5..].to_owned(),
			})
		} else {
			None
		}
	}

	pub fn to_hist_state(&self, add_first: bool) -> String {
		let first_prefix = if add_first { history::HIST_INIT_PREFIX } else { "" };
		match self {
			AppState::Loading => format!("{}{}{}", history::HIST_GAME_PREFIX, first_prefix, "loading"),
			AppState::Login(LoginState::NewUser) => {
				format!("{}{}{}", history::HIST_GAME_PREFIX, first_prefix, "new_user")
			}
			AppState::Login(LoginState::ExistingUser) => {
				format!("{}{}{}", history::HIST_GAME_PREFIX, first_prefix, "existing_user")
			}
			AppState::Login(LoginState::ChooseGameMode) => {
				format!("{}{}{}", history::HIST_GAME_PREFIX, first_prefix, "game_mode")
			}
			AppState::Login(LoginState::OpponentName) => {
				format!("{}{}{}", history::HIST_GAME_PREFIX, first_prefix, "opponent_name")
			}
			AppState::Lobby | AppState::ConnectingToLobby => {
				format!("{}{}{}", history::HIST_GAME_PREFIX, first_prefix, "lobby")
			}
			AppState::InNetGame { game_uid } | AppState::ConnectingToGame { game_uid } => {
				format!(
					"{}{}{}{}",
					history::HIST_GAME_PREFIX,
					first_prefix,
					"in_game_",
					game_uid
				)
			}
			AppState::InLocalGame => format!("{}{}{}", history::HIST_GAME_PREFIX, first_prefix, "in_local_game"),
		}
	}

	pub fn from_hist_state(hist_state: &str) -> Option<AppState> {
		if !hist_state.starts_with(history::HIST_GAME_PREFIX) {
			return None;
		}
		let mut hist_state = &hist_state[history::HIST_GAME_PREFIX.len()..];
		if hist_state.starts_with(history::HIST_INIT_PREFIX) {
			hist_state = &hist_state[history::HIST_INIT_PREFIX.len()..];
		}
		match hist_state {
			"lobby" => Some(AppState::Lobby),
			"opponent_name" => Some(AppState::Login(LoginState::OpponentName)),
			"game_mode" => Some(AppState::Login(LoginState::ChooseGameMode)),
			"existing_user" => Some(AppState::Login(LoginState::ExistingUser)),
			"new_user" => Some(AppState::Login(LoginState::NewUser)),
			"in_local_game" => Some(AppState::InLocalGame),
			"loading" => Some(AppState::Loading),
			_ => {
				if hist_state.starts_with("in_game_") && hist_state.len() > "in_game_".len() {
					Some(AppState::InNetGame {
						game_uid: hist_state["in_game_".len()..].to_owned(),
					})
				} else {
					None
				}
			}
		}
	}

	pub fn title(&self) -> &'static str {
		match self {
			AppState::Loading => "Chargement",
			AppState::Login(LoginState::NewUser)
			| AppState::Login(LoginState::ExistingUser)
			| AppState::Login(LoginState::OpponentName) => "Bienvenu dans le jeu",
			AppState::Login(LoginState::ChooseGameMode) => "Type de connection",
			AppState::Lobby | AppState::ConnectingToLobby => "Choix de la partie",
			AppState::InNetGame { .. } | AppState::ConnectingToGame { .. } => "Partie en ligne",
			AppState::InLocalGame => "Partie locale",
		}
	}

	pub fn url(&self) -> String {
		match self {
			AppState::Loading
			| AppState::Login(LoginState::NewUser)
			| AppState::Login(LoginState::ExistingUser)
			| AppState::Login(LoginState::OpponentName)
			| AppState::Login(LoginState::ChooseGameMode) => "/".to_owned(),
			AppState::Lobby | AppState::ConnectingToLobby => "/lobby".to_owned(),
			AppState::InNetGame { game_uid } | AppState::ConnectingToGame { game_uid } => format!("/game/{}", game_uid),
			AppState::InLocalGame => "/local".to_owned(),
		}
	}
}
