use web_sys::Document;

use crate::utils::dom;

pub struct GameSection {}

impl GameSection {
	pub fn hide(document: &Document) {
		dom::hide_element_by_id(document, "game_section");
	}

	pub fn show(document: &Document) {
		dom::show_element_by_id(document, "game_section");
	}
}

#[derive(Debug, Clone)]
pub struct GameDetails {
	pub id: String,
	pub name: String,
	pub is_finished: bool,
	pub seat_1_username: Option<String>,
	pub seat_2_username: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
	Seat1,
	Seat2,
	Observer,
}

#[derive(Debug, Clone)]
pub enum GameAction {
	Init,
	Move,
	Finished,
}
