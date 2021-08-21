use crate::app::lobby::GameOverview;
use crate::ep::AppEntryPoint;
use crate::ep::EpRef;
use crate::utils::dom;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::HtmlElement;
use web_sys::{Document, Event, HtmlFormElement};

pub struct LobbyForm {
	on_create_cb: Closure<dyn FnMut(Event)>,
}

impl LobbyForm {
	pub fn new(ep: EpRef, document: &Document) -> LobbyForm {
		LobbyForm {
			on_create_cb: LobbyForm::add_create_event_listener(ep, document),
		}
	}

	pub fn show(&self, document: &Document) {
		dom::show_element_by_id(&document, "lobby_section");
	}

	pub fn hide(document: &Document) {
		dom::hide_element_by_id(document, "lobby_section");
	}

	pub fn show_games(&self, document: &Document, game_list: &Vec<GameOverview>) {
		if let Some(element) = document.get_element_by_id("game_list") {
			if let Some(element) = element.dyn_ref::<HtmlElement>() {
				while element.has_child_nodes() {
					if let Some(child) = element.first_child() {
						element.remove_child(&child).ok();
					}
				}

				for game in game_list {
					let child = self.create_game_element(document, game);
					element.append_child(&child).unwrap();
				}
			}
		}
	}

	pub fn upsert_game(&self, document: &Document, game: &GameOverview) {
		if let Some(element) = document.get_element_by_id(format!("lobby_game_{}", game.id).as_str()) {
			if let Some(element) = element.dyn_ref::<HtmlElement>() {
				self.update_game_element(element, game);
			}
		} else if let Some(element) = document.get_element_by_id("game_list") {
			if let Some(element) = element.dyn_ref::<HtmlElement>() {
				let child = self.create_game_element(document, game);
				element.append_child(&child).unwrap();
			}
		}
	}

	pub fn remove_game(&self, document: &Document, game_id: &str) {
		if let Some(element) = document.get_element_by_id(format!("lobby_game_{}", game_id).as_str()) {
			if let Some(element) = element.dyn_ref::<HtmlElement>() {
				element.remove();
			}
		}
	}

	fn create_game_element(&self, document: &Document, game: &GameOverview) -> Element {
		let element = document.create_element("li").unwrap();
		element.set_id(format!("lobby_game_{}", game.id).as_str());
		element.set_class_name("game");

		//FIXME: implement this
		element.set_inner_html(game.name.as_str());
		element
	}

	fn update_game_element(&self, element: &Element, game: &GameOverview) {
		//FIXME: implement this
		element.set_inner_html(game.name.as_str());
	}

	fn add_create_event_listener(ep: EpRef, document: &Document) -> Closure<dyn FnMut(Event)> {
		let handler = move |event: Event| {
			event.prevent_default();
			ep.create_game();
		};
		let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(Event)>);
		dom::by_id!(document, "lobby_form", HtmlFormElement).set_onsubmit(Some(closure.as_ref().unchecked_ref()));
		closure
	}

	fn clean_callbacks(&mut self) {
		if let Some(window) = web_sys::window() {
			if let Some(document) = window.document() {
				dom::by_id!(document, "lobby_form", HtmlFormElement).set_onsubmit(None);
			}
		}
	}
}

impl Drop for LobbyForm {
	fn drop(&mut self) {
		self.clean_callbacks();
	}
}
