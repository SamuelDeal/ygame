use crate::app::game;
use crate::app::lobby;
use crate::app::login::LoginInfo;
use crate::app::state::AppState;
use crate::app::Application;
use crate::conn::Connection;
use crate::err::Error;
use crate::utils::console;
use crate::Result;

use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;
use std::rc::Weak;

use web_sys::Document;
use web_sys::ErrorEvent;
use web_sys::MessageEvent;

pub type EpRef = Rc<RefCell<JsEntryPoint>>;

pub trait LobbyEntryPoint {
	fn on_lobby_game_list(&self, game_list: Vec<lobby::GameOverview>) -> Result<()>;
	fn on_lobby_new_game(&self, info: lobby::GameOverview) -> Result<()>;
	fn on_lobby_game_changed(&self, info: lobby::GameOverview) -> Result<()>;
	fn on_lobby_game_removed(&self, game_id: String) -> Result<()>;
}

pub trait SocketEntryPoint {
	fn on_websocket_reconnect_delay(&self, document: &Document) -> Result<()>;
	fn on_websocket_message(&self, evt: MessageEvent) -> Result<()>;
	fn on_websocket_connected(&self) -> Result<()>;
	fn on_websocket_connection_lost(&self) -> Result<()>;
}

pub trait AppEntryPoint {
	fn login_info(&self) -> Result<LoginInfo>;
	fn set_login_info(&self, info: LoginInfo) -> Result<()>;
	fn on_game_created(&self, info: game::GameDetails, role: game::UserRole) -> Result<()>;
	fn on_game_joined(&self, info: game::GameDetails, role: game::UserRole, moves: Vec<game::GameAction>)
		-> Result<()>;
	fn on_websocket_error(&self, e: ErrorEvent) -> Result<()>;
	fn set_state(&self, state: AppState) -> Result<()>;
	fn create_game(&self) -> Result<()>;
	fn on_connected(&self) -> Result<()>;
	fn on_back_event(&self, state: String) -> Result<()>;
}

#[derive(Clone)]
pub struct JsEntryPoint {
	app_ref: Weak<RefCell<Application>>,
	conn_ref: Weak<Connection>,
}

impl JsEntryPoint {
	pub fn new() -> Self {
		JsEntryPoint {
			app_ref: Weak::new(),
			conn_ref: Weak::new(),
		}
	}

	pub fn set_app(&mut self, app_ref: &Rc<RefCell<Application>>) {
		self.app_ref = Rc::downgrade(app_ref);
	}

	pub fn set_conn(&mut self, conn_ref: &Rc<Connection>) {
		self.conn_ref = Rc::downgrade(conn_ref);
	}
}

#[cfg(debug_assertions)]
macro_rules! borrow_app {
	($self:expr, $x:expr) => {
		$self.with_app($x, stringify!($x))
	};
}

#[cfg(debug_assertions)]
macro_rules! borrow_conn {
	($self:expr, $x:expr) => {
		$self.with_conn($x, stringify!($x))
	};
}

#[cfg(debug_assertions)]
macro_rules! borrow_mut_app {
	($self:expr, $x:expr) => {
		$self.with_mut_app($x, stringify!($x));
	};
}

#[cfg(debug_assertions)]
macro_rules! borrow_mut_app_and_conn {
	($self:expr, $x:expr) => {
		$self.with_mut_app_and_conn($x, stringify!($x))
	};
}

#[cfg(not(debug_assertions))]
macro_rules! borrow_app {
	($self:expr, $x:expr) => {
		$self.with_app($x)
	};
}

#[cfg(not(debug_assertions))]
macro_rules! borrow_mut_app {
	($self:expr, $x:expr) => {
		$self.with_mut_app($x)
	};
}

#[cfg(not(debug_assertions))]
macro_rules! borrow_conn {
	($self:expr, $x:expr) => {
		$self.with_conn($x)
	};
}

#[cfg(not(debug_assertions))]
macro_rules! borrow_mut_app_and_conn {
	($self:expr, $x:expr) => {
		$self.with_mut_app_and_conn($x)
	};
}

impl LobbyEntryPoint for EpRef {
	fn on_lobby_game_list(&self, game_list: Vec<lobby::GameOverview>) -> Result<()> {
		borrow_mut_app!(self, |mut app| app.on_lobby_game_list(game_list))
	}

	fn on_lobby_new_game(&self, info: lobby::GameOverview) -> Result<()> {
		borrow_mut_app!(self, |mut app| app.on_lobby_new_game(info))
	}

	fn on_lobby_game_changed(&self, info: lobby::GameOverview) -> Result<()> {
		borrow_mut_app!(self, |mut app| app.on_lobby_game_changed(info))
	}

	fn on_lobby_game_removed(&self, game_id: String) -> Result<()> {
		borrow_mut_app!(self, |mut app| app.on_lobby_game_removed(game_id))
	}
}

impl SocketEntryPoint for EpRef {
	fn on_websocket_reconnect_delay(&self, document: &Document) -> Result<()> {
		borrow_conn!(self, |conn| conn.on_reconnect_delay(document))
	}

	fn on_websocket_message(&self, evt: MessageEvent) -> Result<()> {
		borrow_mut_app_and_conn!(self, |mut app, conn| conn.on_message(evt, &mut app))
	}

	fn on_websocket_connected(&self) -> Result<()> {
		borrow_mut_app_and_conn!(self, |mut app, conn| conn.on_connected(&mut app))
	}

	fn on_websocket_connection_lost(&self) -> Result<()> {
		borrow_conn!(self, |conn| conn.on_connection_lost())
	}
}

impl AppEntryPoint for EpRef {
	fn login_info(&self) -> Result<LoginInfo> {
		borrow_mut_app!(self, |app| app.login_info())
	}

	fn set_login_info(&self, info: LoginInfo) -> Result<()> {
		borrow_mut_app!(self, |mut app| app.set_login_info(info))
	}

	fn on_game_created(&self, info: game::GameDetails, role: game::UserRole) -> Result<()> {
		borrow_mut_app!(self, |mut app| app.on_game_created(info, role))
	}

	fn on_game_joined(
		&self,
		info: game::GameDetails,
		role: game::UserRole,
		moves: Vec<game::GameAction>,
	) -> Result<()> {
		borrow_mut_app!(self, |mut app| app.on_game_joined(info, role, moves))
	}

	fn on_websocket_error(&self, e: ErrorEvent) -> Result<()> {
		borrow_mut_app_and_conn!(self, |mut app, conn| app.on_socket_error(e, conn))
	}

	fn set_state(&self, state: AppState) -> Result<()> {
		borrow_mut_app_and_conn!(self, |mut app, conn| app.set_state(state, conn))
	}

	fn create_game(&self) -> Result<()> {
		borrow_mut_app!(self, |mut app| app.create_game())
	}

	fn on_connected(&self) -> Result<()> {
		borrow_mut_app!(self, |mut app| app.on_connected())
	}

	fn on_back_event(&self, state: String) -> Result<()> {
		borrow_mut_app_and_conn!(self, |mut app, conn| app.on_history_back(state, conn))
	}
}

#[cfg(debug_assertions)]
trait DelegateApp {
	fn with_conn<T, F>(&self, f: F, debug_str: &str) -> Result<T>
	where
		F: FnOnce(&Connection) -> T;

	fn with_app<T, F>(&self, f: F, debug_str: &str) -> Result<T>
	where
		F: FnOnce(Ref<Application>) -> T;

	fn with_mut_app<T, F>(&self, f: F, debug_str: &str) -> Result<T>
	where
		F: FnOnce(RefMut<Application>) -> T;

	fn with_mut_app_and_conn<T, F>(&self, f: F, debug_str: &str) -> Result<T>
	where
		F: FnOnce(RefMut<Application>, &Connection) -> T;
}

#[cfg(debug_assertions)]
impl DelegateApp for EpRef {
	fn with_conn<T, F>(&self, f: F, debug_str: &str) -> Result<T>
	where
		F: FnOnce(&Connection) -> T,
	{
		console::error!("START borrow conn here: {}", debug_str);
		let res = match self.try_borrow() {
			Err(e) => Err(Error::ImplError {
				details: "Unable to borrow mutable reference to the entry_point".to_owned(),
				source: Some(Box::new(e)),
			}),
			Ok(ep) => match ep.conn_ref.upgrade() {
				None => Err(Error::ImplError {
					details: "The connection is not reachable anymore".to_owned(),
					source: None,
				}),
				Some(conn) => Ok(f(conn.as_ref())),
			},
		};
		if let Err(e) = &res {
			console::error!("ERROR DETECTED on {}: {:?}", debug_str, e);
		}
		console::error!("END borrow conn: {}", debug_str);
		res
	}

	fn with_app<T, F>(&self, f: F, debug_str: &str) -> Result<T>
	where
		F: FnOnce(Ref<Application>) -> T,
	{
		console::error!("START borrow app here: {}", debug_str);
		let res = match self.try_borrow() {
			Err(e) => Err(Error::ImplError {
				details: "Unable to borrow mutable reference to the entry_point".to_owned(),
				source: Some(Box::new(e)),
			}),
			Ok(ep) => match ep.app_ref.upgrade() {
				None => Err(Error::ImplError {
					details: "The application is not reachable anymore".to_owned(),
					source: None,
				}),
				Some(app) => match app.try_borrow() {
					Err(e) => Err(Error::ImplError {
						details: "Unable to borrow const reference to the application".to_owned(),
						source: Some(Box::new(e)),
					}),
					Ok(app) => Ok(f(app)),
				},
			},
		};
		if let Err(e) = &res {
			console::error!("ERROR DETECTED on {}: {:?}", debug_str, e);
		}
		console::error!("END borrow app: {}", debug_str);
		res
	}

	fn with_mut_app<T, F>(&self, f: F, debug_str: &str) -> Result<T>
	where
		F: FnOnce(RefMut<Application>) -> T,
	{
		console::error!("START MUT borrow app here: {}", debug_str);
		let res = match self.try_borrow() {
			Err(e) => Err(Error::ImplError {
				details: "Unable to borrow mutable reference to the entry_point".to_owned(),
				source: Some(Box::new(e)),
			}),
			Ok(ep) => match ep.app_ref.upgrade() {
				None => Err(Error::ImplError {
					details: "The application is not reachable anymore".to_owned(),
					source: None,
				}),
				Some(app) => match app.try_borrow_mut() {
					Err(e) => Err(Error::ImplError {
						details: "Unable to borrow mutable reference to the application".to_owned(),
						source: Some(Box::new(e)),
					}),
					Ok(app) => Ok(f(app)),
				},
			},
		};
		if let Err(e) = &res {
			console::error!("ERROR DETECTED on {}: {:?}", debug_str, e);
		}
		console::error!("END MUT borrow app: {}", debug_str);
		res
	}

	fn with_mut_app_and_conn<T, F>(&self, f: F, debug_str: &str) -> Result<T>
	where
		F: FnOnce(RefMut<Application>, &Connection) -> T,
	{
		console::error!("START MUT borrow app here: {}", debug_str);
		let res = match self.try_borrow() {
			Err(e) => Err(Error::ImplError {
				details: "Unable to borrow mutable reference to the entry_point".to_owned(),
				source: Some(Box::new(e)),
			}),
			Ok(ep) => match ep.app_ref.upgrade() {
				None => Err(Error::ImplError {
					details: "The application is not reachable anymore".to_owned(),
					source: None,
				}),
				Some(app) => match app.try_borrow_mut() {
					Err(e) => Err(Error::ImplError {
						details: "Unable to borrow mutable reference to the application".to_owned(),
						source: Some(Box::new(e)),
					}),
					Ok(app) => match ep.conn_ref.upgrade() {
						None => Err(Error::ImplError {
							details: "The connection is not reachable anymore".to_owned(),
							source: None,
						}),
						Some(conn) => Ok(f(app, conn.as_ref())),
					},
				},
			},
		};
		if let Err(e) = &res {
			console::error!("ERROR DETECTED on {}: {:?}", debug_str, e);
		}
		console::error!("END MUT borrow app: {}", debug_str);
		res
	}
}

#[cfg(not(debug_assertions))]
trait DelegateApp {
	fn with_conn<T, F>(&self, f: F) -> Result<T>
	where
		F: FnOnce(&Connection) -> T;

	fn with_app<T, F>(&self, f: F) -> Result<T>
	where
		F: FnOnce(Ref<Application>) -> T;

	fn with_mut_app<T, F>(&self, f: F) -> Result<T>
	where
		F: FnOnce(RefMut<Application>) -> T;

	fn with_mut_app_and_conn<T, F>(&self, f: F) -> Result<T>
	where
		F: FnOnce(RefMut<Application>, &Connection) -> T;
}

#[cfg(not(debug_assertions))]
impl DelegateApp for EpRef {
	fn with_conn<T, F>(&self, f: F) -> Result<T>
	where
		F: FnOnce(&Connection) -> T,
	{
		match self.try_borrow() {
			Err(e) => Err(Error::ImplError {
				details: "Unable to borrow mutable reference to the entry_point".to_owned(),
				source: Some(Box::new(e)),
			}),
			Ok(ep) => match ep.conn_ref.upgrade() {
				None => Err(Error::ImplError {
					details: "The connection is not reachable anymore".to_owned(),
					source: None,
				}),
				Some(conn) => Ok(f(conn.as_ref())),
			},
		}
	}

	fn with_app<T, F>(&self, f: F) -> Result<T>
	where
		F: FnOnce(Ref<Application>) -> T,
	{
		match self.try_borrow() {
			Err(e) => Err(Error::ImplError {
				details: "Unable to borrow mutable reference to the entry_point".to_owned(),
				source: Some(Box::new(e)),
			}),
			Ok(ep) => match ep.app_ref.upgrade() {
				None => Err(Error::ImplError {
					details: "The application is not reachable anymore".to_owned(),
					source: None,
				}),
				Some(app) => match app.try_borrow() {
					Err(e) => Err(Error::ImplError {
						details: "Unable to borrow const reference to the application".to_owned(),
						source: Some(Box::new(e)),
					}),
					Ok(app) => Ok(f(app)),
				},
			},
		}
	}

	fn with_mut_app<T, F>(&self, f: F) -> Result<T>
	where
		F: FnOnce(RefMut<Application>) -> T,
	{
		match self.try_borrow() {
			Err(e) => Err(Error::ImplError {
				details: "Unable to borrow mutable reference to the entry_point".to_owned(),
				source: Some(Box::new(e)),
			}),
			Ok(ep) => match ep.app_ref.upgrade() {
				None => Err(Error::ImplError {
					details: "The application is not reachable anymore".to_owned(),
					source: None,
				}),
				Some(app) => match app.try_borrow_mut() {
					Err(e) => Err(Error::ImplError {
						details: "Unable to borrow mutable reference to the application".to_owned(),
						source: Some(Box::new(e)),
					}),
					Ok(app) => Ok(f(app)),
				},
			},
		}
	}

	fn with_mut_app_and_conn<T, F>(&self, f: F) -> Result<T>
	where
		F: FnOnce(RefMut<Application>, &Connection) -> T,
	{
		match self.try_borrow() {
			Err(e) => Err(Error::ImplError {
				details: "Unable to borrow mutable reference to the entry_point".to_owned(),
				source: Some(Box::new(e)),
			}),
			Ok(ep) => match ep.app_ref.upgrade() {
				None => Err(Error::ImplError {
					details: "The application is not reachable anymore".to_owned(),
					source: None,
				}),
				Some(app) => match app.try_borrow_mut() {
					Err(e) => Err(Error::ImplError {
						details: "Unable to borrow mutable reference to the application".to_owned(),
						source: Some(Box::new(e)),
					}),
					Ok(app) => match ep.conn_ref.upgrade() {
						None => Err(Error::ImplError {
							details: "The connection is not reachable anymore".to_owned(),
							source: None,
						}),
						Some(conn) => Ok(f(app, conn.as_ref())),
					},
				},
			},
		}
	}
}
