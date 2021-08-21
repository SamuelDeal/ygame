pub mod game;
pub mod lobby;
pub mod login;
pub mod state;

use crate::app::game::GameSection;
use crate::app::lobby::form::LobbyForm;
use crate::app::login::form::LoginForm;
use crate::app::login::LoginInfo;
use crate::app::state::AppState;
use crate::app::state::LoginState;
use crate::conn;
use crate::conn::loader;
use crate::conn::Connection;
use crate::ep::AppEntryPoint;
use crate::ep::EpRef;
use crate::utils::browser;
use crate::utils::console;
use crate::utils::history;

use std::collections::BTreeMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::ErrorEvent;
use web_sys::Window;
use web_sys::{Document, PopStateEvent};

/// This object represent the main application state
///
/// It manage the html, the history and the current game
pub struct Application {
	entry_point: EpRef,
	login_form: LoginForm,
	lobby_form: LobbyForm,
	subfolder: Option<String>,
	login: LoginInfo,
	current_state: AppState,
	desired_state: Option<AppState>,
	rolling_back_history: bool,
	is_fully_connected: bool,
	lobby_games: BTreeMap<String, lobby::GameOverview>,
	on_back_cb: Closure<dyn FnMut(PopStateEvent)>,
}

impl Drop for Application {
	fn drop(&mut self) {
		console::error!("DROPPING APP !");
		if let Some(window) = web_sys::window() {
			window.set_onpopstate(None);
		}
		console::error!("Application state cleaned !");
	}
}

impl Application {
	pub fn new(window: &Window, document: &Document, subfolder: Option<String>, entry_point: EpRef) -> Self {
		let entry_point_clone = entry_point.clone();
		let handler = move |event: PopStateEvent| {
			let state_value: JsValue = event.state();
			let state = state_value.as_string().unwrap_or_default();
			console::error!("OnPopState {}", state);
			if state.is_empty() {
				return;
			}
			event.prevent_default();
			entry_point_clone.on_back_event(state);
		};
		let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(PopStateEvent)>);
		window.set_onpopstate(Some(closure.as_ref().unchecked_ref()));

		let hist_state = history::get_current_hist_state(window);
		let desired_state = browser::get_url_path(&document)
			.map(|url| AppState::from_url(url, &subfolder))
			.flatten();
		let rolling_back_history = if let Some(hist_state) = hist_state {
			history::is_game_state(&hist_state) && !history::is_first_game_state(&hist_state)
		} else {
			false
		};

		Application {
			entry_point: entry_point.clone(),
			login_form: LoginForm::new(entry_point.clone(), &document),
			lobby_form: LobbyForm::new(entry_point.clone(), &document),
			subfolder,
			current_state: AppState::Loading,
			desired_state,
			rolling_back_history,
			login: LoginInfo::load(&window),
			is_fully_connected: false,
			lobby_games: BTreeMap::new(),
			on_back_cb: closure,
		}
	}

	pub fn start(&mut self, conn: &Connection) {
		console::log!("START APP MODIFIED !");
		if self.rolling_back_history {
			self.go_back();
		} else {
			self.on_start(conn);
		}
	}

	fn on_start(&mut self, conn: &Connection) {
		console::log!("on_start!");
		let initial_state = match self.login {
			LoginInfo::None => AppState::Login(LoginState::NewUser),
			LoginInfo::NewUser { .. } | LoginInfo::ExistingUser { .. } => AppState::Login(LoginState::ExistingUser),
		};
		self.set_first_state(initial_state, conn);
	}

	pub fn set_login_info(&mut self, login: LoginInfo) {
		let window = web_sys::window().expect("no global `window` exists");
		login.save(&window);

		self.login = login;
	}

	pub fn on_history_back(&mut self, state: String, conn: &Connection) {
		console::log!("Back event found: {:?}", state);

		if self.rolling_back_history {
			if history::is_first_game_state(&state) {
				self.rolling_back_history = false;
				self.on_start(conn);
			} else {
				self.go_back();
			}
		} else {
			let hist_state = AppState::from_hist_state(&state).unwrap_or(AppState::Loading);
			self.transition_to_state(hist_state, conn);
		}
	}

	pub fn on_socket_error(&mut self, e: ErrorEvent, conn: &Connection) {
		match self.current_state {
			AppState::InNetGame { .. } | AppState::Lobby => {
				self.go_back();
			}
			_ => {}
		}
		conn.on_socket_error(e);
	}

	pub fn login_info(&self) -> LoginInfo {
		self.login.clone()
	}

	pub fn create_game(&mut self) {
		//self.conn.create_game()
	}

	pub fn on_lobby_game_list(&mut self, game_list: Vec<lobby::GameOverview>) {
		if self.current_state == AppState::Lobby {
			let window = web_sys::window().expect("no global `window` exists");
			let document = window.document().expect("window has not document");

			self.lobby_form.show(&document);
			self.lobby_form.show_games(&document, &game_list);
		}
		let mut new_map = BTreeMap::new();
		for game in game_list {
			new_map.insert(game.id.clone(), game);
		}
		self.lobby_games = new_map;
	}

	pub fn on_lobby_new_game(&mut self, info: lobby::GameOverview) {
		if self.current_state == AppState::Lobby {
			let window = web_sys::window().expect("no global `window` exists");
			let document = window.document().expect("window has not document");
			self.lobby_form.upsert_game(&document, &info);
		}
		self.lobby_games.insert(info.id.clone(), info);
	}

	pub fn on_lobby_game_changed(&mut self, info: lobby::GameOverview) {
		if self.current_state == AppState::Lobby {
			let window = web_sys::window().expect("no global `window` exists");
			let document = window.document().expect("window has not document");

			self.lobby_form.upsert_game(&document, &info);
		}
		self.lobby_games.insert(info.id.clone(), info);
	}

	pub fn on_lobby_game_removed(&mut self, game_id: String) {
		if self.current_state == AppState::Lobby {
			let window = web_sys::window().expect("no global `window` exists");
			let document = window.document().expect("window has not document");
			self.lobby_form.remove_game(&document, &game_id);
		}
		self.lobby_games.remove(&game_id);
	}

	pub fn on_game_created(&mut self, info: game::GameDetails, role: game::UserRole) {
		unimplemented!()
	}

	pub fn on_game_joined(&mut self, info: game::GameDetails, role: game::UserRole, moves: Vec<game::GameAction>) {
		unimplemented!()
	}

	pub fn on_connected(&mut self) {
		self.is_fully_connected = true;
		let window = web_sys::window().expect("no global `window` exists");
		let document = window.document().expect("window has not document");
		match &self.current_state {
			AppState::Lobby => {
				loader::hide(&document);
				self.lobby_form.show(&document);
			}
			AppState::InNetGame { game_uid } => {
				loader::hide(&document);
				unimplemented!("coucou")
			}
			_ => (),
		}
	}

	pub fn set_state(&mut self, state: AppState, conn: &Connection) {
		console::log!("set state {:?}", state);
		self.push_state(state.clone());
		self.transition_to_state(state, conn);
	}

	fn set_first_state(&mut self, state: AppState, conn: &Connection) {
		console::log!("set state {:?}", state);
		self.push_any_state(state.clone(), true);
		self.transition_to_state(state, conn);
	}

	fn push_state(&mut self, state: AppState) {
		self.push_any_state(state, false)
	}

	fn push_any_state(&mut self, state: AppState, is_first: bool) {
		let window = web_sys::window().expect("no global `window` exists");
		let document = window.document().expect("document in window");
		let history = window.history().expect("no window `history` exists");

		let state_jsstr = JsValue::from(state.to_hist_state(is_first));
		let state_url = state.url();
		let state_url = match &self.subfolder {
			Some(subfolder) => format!("/{}{}", subfolder, state_url),
			None => state_url,
		};
		let state_url = browser::to_full_url(&document, &state_url);
		if is_first {
			history
				.replace_state_with_url(&state_jsstr, state.title(), Some(&state_url))
				.ok();
		} else {
			history
				.push_state_with_url(&state_jsstr, state.title(), Some(&state_url))
				.ok();
		}
	}

	fn transition_to_state(&mut self, next_state: AppState, conn: &Connection) {
		if next_state == self.current_state {
			console::log!("Same state, skipping {:?}", next_state);
			return;
		}
		console::log!("going to state {:?}", next_state);
		let old_state = self.current_state.clone();

		self.on_leave_state(&old_state, &next_state, conn);
		self.current_state = next_state.clone();
		self.on_enter_state(&old_state, &next_state, conn);
	}

	fn on_leave_state(&mut self, old_state: &AppState, next_state: &AppState, conn: &Connection) {
		let window = web_sys::window().expect("no global `window` exists");
		let document = window.document().expect("no `document` in window");
		match old_state {
			AppState::Loading => {
				conn::loader::hide(&document);
			}
			AppState::Login(..) => match next_state {
				AppState::Login(..) => {}
				_ => {
					LoginForm::hide(&document);
				}
			},
			AppState::ConnectingToLobby | AppState::ConnectingToGame { .. } => {
				conn::loader::hide(&document);
			}
			AppState::Lobby => {
				LobbyForm::hide(&document);
			}
			AppState::InNetGame { .. } | AppState::InLocalGame => {
				GameSection::hide(&document);
			}
		}
		match next_state {
			AppState::Loading | AppState::Login(..) | AppState::InLocalGame => {
				self.disconnect(conn);
			}
			_ => {}
		}
	}

	fn on_enter_state(&mut self, _old_state: &AppState, next_state: &AppState, conn: &Connection) {
		let window = web_sys::window().expect("no global `window` exists");
		let document = window.document().expect("no `document` in window");

		match next_state {
			AppState::Loading | AppState::Login(..) | AppState::InLocalGame => {}
			_ => {
				self.connect_to_server(conn, &document);
			}
		}

		match next_state {
			AppState::Login(login_state) => {
				self.login_form.show_state(&self.login, login_state);
			}
			AppState::Loading => {
				conn::loader::show_loading(&document);
			}
			AppState::ConnectingToLobby | AppState::ConnectingToGame { .. } => {
				conn::loader::show_connecting(&document);
			}
			AppState::Lobby => {
				self.lobby_form.show(&document);
			}
			AppState::InNetGame { .. } => GameSection::show(&document),
			AppState::InLocalGame => GameSection::show(&document),
		}
	}

	fn connect_to_server(&mut self, conn: &Connection, document: &Document) {
		if self.is_fully_connected {
			self.on_connected();
			return;
		} else {
			conn.connect(document);
		}
	}

	fn disconnect(&mut self, conn: &Connection) {
		self.is_fully_connected = false;
		conn.disconnect()
	}

	fn ep(&self) -> EpRef {
		self.entry_point.clone()
	}

	fn go_back(&mut self) {
		let window = web_sys::window().expect("no global `window` exists");
		let history = window.history().expect("no window `history` exists");
		history.back();
	}
}
