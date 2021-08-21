mod name;

use std::{
	collections::{BTreeMap, BTreeSet},
	fmt::{self, Display},
	time::{Duration, Instant},
};

use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Recipient, Running};
use rand::{prelude::SliceRandom, Rng};
use uuid::Uuid;

use crate::actor_msg as actmsg;
use crate::client;
use crate::error::*;
use crate::lobby;
use crate::log;
use crate::user;
use ygame_game as rules;

//use serde::{Deserialize, Serialize};

const GAME_EXPIRATION: Duration = Duration::from_secs(3600 * 24 * 30);

#[derive(Debug)]
pub struct Game {
	pub id: Uuid,
	pub name: String,
	inited: bool,
	finished: bool,
	timeout: Instant,
	lobby_addr: Addr<lobby::Lobby>,
	seat_1_user_uid: Option<Uuid>,
	seat_1_username: Option<String>,
	seat_2_user_uid: Option<Uuid>,
	seat_2_username: Option<String>,
	clients: BTreeMap<Uuid, Recipient<actmsg::GameMessage>>,
	users: BTreeMap<Uuid, BTreeSet<Uuid>>, // clients ids indexed by user id
	moves: Vec<rules::Action>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
	Created,
	Started,
	Finished,
}

#[derive(Debug, Clone)]
pub struct Info {
	pub id: Uuid,
	pub name: String,
	pub status: Status,
	pub seat_1: Option<Uuid>,
	pub seat_2: Option<Uuid>,
}

impl Actor for Game {
	type Context = Context<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {
		ctx.run_interval(Duration::from_secs(60), |act, ctx| {
			if Instant::now() > act.timeout {
				ctx.stop();
			}
		});
	}

	fn stopping(&mut self, _: &mut Self::Context) -> Running {
		self.lobby_addr.do_send(actmsg::GameClosed { game_id: self.id });
		Running::Stop
	}
}

impl Game {
	pub fn new(lobby_addr: Addr<lobby::Lobby>) -> Game {
		Game {
			id: Uuid::new_v4(),
			inited: false,
			finished: false,
			timeout: Instant::now() + GAME_EXPIRATION,
			lobby_addr,
			name: name::generate_name(),
			seat_1_user_uid: None,
			seat_1_username: None,
			seat_2_user_uid: None,
			seat_2_username: None,
			clients: BTreeMap::new(),
			users: BTreeMap::new(),
			moves: Vec::new(),
		}
	}

	pub fn client_join(
		&mut self,
		user_uid: Uuid,
		username: &str,
		client_id: Uuid,
		client_addr: Recipient<actmsg::GameMessage>,
	) -> rules::UserRole {
		let seat = self.choose_seat(user_uid);
		log::info!("Client joined game {} as {}", self.name, seat);
		if seat == rules::UserRole::Seat1 {
			self.seat_1_user_uid = Some(user_uid);
			self.seat_1_username = Some(username.to_owned());
		} else if seat == rules::UserRole::Seat2 {
			self.seat_2_user_uid = Some(user_uid);
			self.seat_2_username = Some(username.to_owned());
		}
		let client_set = self.users.entry(user_uid).or_insert(BTreeSet::new());
		client_set.insert(client_id);
		self.clients.insert(client_id, client_addr);

		seat
	}

	pub fn info(&self) -> Info {
		Info {
			id: self.id,
			name: self.name.clone(),
			status: if self.finished {
				Status::Finished
			} else if self.inited {
				Status::Started
			} else {
				Status::Created
			},
			seat_1: self.seat_1_user_uid,
			seat_2: self.seat_2_user_uid,
		}
	}

	fn choose_seat(&self, user_uid: Uuid) -> rules::UserRole {
		if Some(user_uid) == self.seat_1_user_uid {
			return rules::UserRole::Seat1;
		}
		if Some(user_uid) == self.seat_2_user_uid {
			return rules::UserRole::Seat2;
		}
		let mut available_seats: Vec<rules::UserRole> = Vec::new();
		if self.seat_1_user_uid.is_none() {
			available_seats.push(rules::UserRole::Seat1);
		}
		if self.seat_2_user_uid.is_none() {
			available_seats.push(rules::UserRole::Seat2);
		}
		available_seats
			.choose(&mut rand::thread_rng())
			.unwrap_or(&rules::UserRole::Observer)
			.clone()
	}

	// Return the uuid of the quitting user if it as no more client connected
	fn client_quit(&mut self, client_id: &Uuid) -> Option<Uuid> {
		let mut disconnect_user_uid = None;
		self.clients.remove(client_id);
		for (user_uid, client_set) in &mut self.users {
			client_set.remove(client_id);
			if client_set.is_empty() {
				disconnect_user_uid = Some(user_uid.clone());
			}
		}
		self.users.retain(|_, v| !v.is_empty());
		disconnect_user_uid
	}

	fn get_user_role(&self, user_uid: Uuid) -> rules::UserRole {
		if Some(user_uid) == self.seat_1_user_uid {
			rules::UserRole::Seat1
		} else if Some(user_uid) == self.seat_1_user_uid {
			rules::UserRole::Seat2
		} else {
			rules::UserRole::Observer
		}
	}

	fn init_game(&mut self) {
		let action = rules::Action::Init;
		self.moves.push(action.clone());
		self.inited = true;
		let msg = actmsg::GameMessage {
			game_id: self.id,
			details: actmsg::GameMessageDetails::Action(action),
		};
		for (_, client) in &self.clients {
			if let Err(e) = client.do_send(msg.clone()) {
				log::warning!("Unable to send message to client: {}", e);
			}
		}
	}
}

impl Handler<actmsg::GameAction> for Game {
	type Result = actmsg::GameActionResponse;

	fn handle(&mut self, msg: actmsg::GameAction, _: &mut Context<Self>) -> Self::Result {
		unimplemented!("HERE !!!")
	}
}

impl Handler<actmsg::JoinGame> for Game {
	type Result = actmsg::GameJoined;

	fn handle(&mut self, msg: actmsg::JoinGame, ctx: &mut Context<Self>) -> Self::Result {
		let username = msg.user_name.clone();

		// Add client to client list and define user role
		let seat = self.client_join(
			msg.user_uid,
			username.clone().as_str(),
			msg.client_id,
			msg.client_addr.clone(),
		);

		// Tell other clients that someone has joined
		ctx.run_later(Duration::from_millis(1), move |this, _| {
			let client_msg = actmsg::GameMessage {
				game_id: this.id,
				details: actmsg::GameMessageDetails::UserJoin {
					user_uid: msg.user_uid,
					user_name: username.clone(),
					user_seat: seat.clone(),
				},
			};
			for (client_id, client) in &this.clients {
				if *client_id == msg.client_id {
					continue;
				}
				if let Err(e) = client.do_send(client_msg.clone()) {
					log::warning!("Unable to send message to client: {}", e);
				}
			}
		});

		// Init the game and tell all game clients that the game has been inited
		if self.seat_1_user_uid.is_some() && self.seat_2_user_uid.is_some() && !self.inited {
			ctx.run_later(Duration::from_millis(1), |this, _| {
				this.init_game();
			});
		}

		// Return the current game info
		actmsg::GameJoined {
			game_name: self.name.clone(),
			user_role: seat,
			seat_1_username: self.seat_1_username.clone(),
			seat_2_username: self.seat_2_username.clone(),
			moves: self.moves.clone(),
		}
	}
}

impl Handler<actmsg::Disconnect> for Game {
	type Result = ();

	fn handle(&mut self, msg: actmsg::Disconnect, _: &mut Context<Self>) -> Self::Result {
		//Remove client from list of connected clients, and check if it's the last client of a user
		let disconnect_user_uid = self.client_quit(&msg.client_id);

		// Send to
		if let Some(user_uid) = disconnect_user_uid {
			let user_seat = self.get_user_role(user_uid);
			let msg = actmsg::GameMessage {
				game_id: self.id,
				details: actmsg::GameMessageDetails::UserQuit { user_uid, user_seat },
			};
			for (_, client) in &self.clients {
				if let Err(e) = client.do_send(msg.clone()) {
					log::warning!("Unable to send message to client: {}", e);
				}
			}
		}
	}
}
