use crate::actor_msg as actmsg;
use crate::error::*;
use crate::game;
use crate::log;

use std::collections::{BTreeMap, BTreeSet};
use std::iter::Map;
use std::ops::Deref;
use std::time::{Duration, Instant};

use actix::prelude::*;
use uuid::Uuid;

const SESSION_DURATION: Duration = Duration::from_secs(3600 * 24 * 30);

#[derive(Debug)]
pub struct Lobby {
	sessions: BTreeMap<Uuid, (Uuid, Instant)>, // (User uid, timeout) indexed by session uid
	clients: BTreeMap<Uuid, Recipient<actmsg::LobbyMessage>>, // Client addr indexed by client id
	clients_by_users: BTreeMap<Uuid, BTreeSet<Uuid>>, //Client ids by user uid
	user_by_client: BTreeMap<Uuid, Uuid>,      //user id by client id
	games: BTreeMap<Uuid, (game::Info, Addr<game::Game>)>, //games by game_id
}

impl Actor for Lobby {
	type Context = Context<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {
		ctx.run_interval(Duration::from_secs(60), |this, _| {
			this.clean_old_sessions();
		});
	}
}

impl Lobby {
	pub fn new() -> Lobby {
		Lobby {
			sessions: BTreeMap::new(),
			clients: BTreeMap::new(),
			clients_by_users: BTreeMap::new(),
			user_by_client: BTreeMap::new(),
			games: BTreeMap::new(),
		}
	}

	fn clean_old_sessions(&mut self) {
		let now = Instant::now();
		self.sessions.retain(move |_, (_, timeout)| timeout.deref() > &now);
		log::info!("After clean: {} sessions remaining", self.sessions.len())
	}

	fn set_session(&mut self, user_uid: Option<Uuid>, session_uid: Option<Uuid>) -> (Uuid, Uuid) {
		if let (Some(session_uid), Some(msg_user_uid)) = (session_uid, user_uid) {
			if let Some((user_uid, timeout)) = self.sessions.get_mut(&session_uid) {
				if msg_user_uid == *user_uid {
					*timeout = Instant::now() + SESSION_DURATION;
					log::debug!("Restored session {} for user {}", session_uid, user_uid);
					return (msg_user_uid, session_uid);
				}
			}
		}
		let new_session_uid = Uuid::new_v4();
		let new_user_uid = Uuid::new_v4();
		self.sessions
			.insert(new_session_uid, (new_user_uid, Instant::now() + SESSION_DURATION));
		log::debug!("Session {} created for user {}", new_session_uid, new_user_uid);
		return (new_user_uid, new_session_uid);
	}

	fn save_user_client_asso(&mut self, user_uid: Uuid, client_id: Uuid) {
		//Clean old association if already exists
		if let Some(_) = self.user_by_client.get(&client_id) {
			if let Some(client_list) = self.clients_by_users.get_mut(&user_uid) {
				client_list.remove(&client_id);
			}
			self.user_by_client.remove(&client_id);
		}

		//Save new association
		let user_list = self.clients_by_users.entry(user_uid).or_insert(BTreeSet::new());
		user_list.insert(client_id);
		self.user_by_client.insert(client_id, user_uid);
	}

	fn on_disconnect(&mut self, msg: actmsg::Disconnect, _: &mut Context<Self>) {
		log::info!("User disconnected");

		self.games.values().for_each(|(_, addr)| {
			if let Err(e) = addr.clone().recipient().do_send(msg.clone()) {
				log::warning!("Unable to send message to client: {}", e);
			}
		});
		self.clients.remove(&msg.client_id);
		if let Some(user_uid) = self.user_by_client.remove(&msg.client_id) {
			if let Some(client_list) = self.clients_by_users.get_mut(&user_uid) {
				client_list.remove(&msg.client_id);
			}
		}
	}
}

impl Handler<actmsg::RegisterUserMessage> for Lobby {
	type Result = actmsg::RegisterUserResponse;

	fn handle(&mut self, msg: actmsg::RegisterUserMessage, _: &mut Context<Self>) -> Self::Result {
		let (user_uid, session_uid) = self.set_session(msg.user_uid, msg.session_uid);
		self.save_user_client_asso(user_uid, msg.client_id);
		actmsg::RegisterUserResponse {
			name: msg.name,
			session_uid: session_uid,
			user_uid: user_uid,
		}
	}
}

impl Handler<actmsg::AskGameList> for Lobby {
	type Result = actmsg::GameList;

	fn handle(&mut self, _: actmsg::AskGameList, _: &mut Context<Self>) -> Self::Result {
		actmsg::GameList {
			list: self.games.values().map(|(info, _)| info.clone()).collect(),
		}
	}
}

impl Handler<actmsg::CreateGame> for Lobby {
	type Result = actmsg::GameCreated;

	fn handle(&mut self, msg: actmsg::CreateGame, ctx: &mut Context<Self>) -> Self::Result {
		let mut game = game::Game::new(ctx.address());
		let user_seat = game.client_join(msg.user_uid, msg.user_name.as_str(), msg.client_id, msg.client_addr);
		let game_id = game.id;
		let game_name = game.name.clone();
		let game_info = game.info();
		let game_addr = game.start();
		self.games.insert(game_id, (game_info.clone(), game_addr.clone()));
		ctx.run_later(Duration::from_millis(1), |this, _| {
			let msg = actmsg::LobbyMessage::NewGame(game_info);
			for (_, client) in &this.clients {
				if let Err(e) = client.do_send(msg.clone()) {
					log::warning!("Unable to send message to client: {}", e);
				}
			}
		});
		actmsg::GameCreated {
			game_uid: game_id,
			game_name: game_name,
			game_addr: game_addr.recipient(),
			user_seat,
		}
	}
}

impl Handler<actmsg::GetGame> for Lobby {
	type Result = actmsg::GameFound;

	fn handle(&mut self, msg: actmsg::GetGame, _: &mut Context<Self>) -> Self::Result {
		match self.games.get(&msg.game_id) {
			None => actmsg::GameFound::Failure,
			Some((_, game_addr)) => actmsg::GameFound::Success {
				game: game_addr.clone(),
			},
		}
	}
}

impl Handler<actmsg::GameClosed> for Lobby {
	type Result = ();

	fn handle(&mut self, msg: actmsg::GameClosed, _: &mut Context<Self>) -> Self::Result {
		if let Some(_) = self.games.remove(&msg.game_id) {
			let msg = actmsg::LobbyMessage::GameRemoved { game_id: msg.game_id };
			for (_, client) in &self.clients {
				if let Err(e) = client.do_send(msg.clone()) {
					log::warning!("Unable to send message to client: {}", e);
				}
			}
		}
	}
}

impl Handler<actmsg::Connect> for Lobby {
	type Result = ();

	fn handle(&mut self, msg: actmsg::Connect, _: &mut Context<Self>) -> Self::Result {
		log::info!("User connected");
		self.clients.remove(&msg.client_id);
		self.clients.insert(msg.client_id, msg.client_addr);
	}
}

impl Handler<actmsg::Disconnect> for Lobby {
	type Result = ();

	fn handle(&mut self, msg: actmsg::Disconnect, ctx: &mut Context<Self>) -> Self::Result {
		self.games.values().for_each(|(_, addr)| addr.do_send(msg.clone()));
		self.on_disconnect(msg, ctx);
	}
}
