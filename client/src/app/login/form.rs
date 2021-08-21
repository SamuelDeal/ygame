use crate::app::login::LoginInfo;
use crate::app::state::AppState;
use crate::app::state::LoginState;
use crate::ep::AppEntryPoint;
use crate::ep::EpRef;
use crate::utils::console;
use crate::utils::dom;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Event, HtmlElement, HtmlFormElement};

pub struct LoginForm {
	on_new_user_cb: Closure<dyn FnMut(Event)>,
	on_forget_user_cb: Closure<dyn FnMut(Event)>,
	on_existing_user_cb: Closure<dyn FnMut(Event)>,
	on_local_game_cb: Closure<dyn FnMut()>,
	on_net_game_cb: Closure<dyn FnMut()>,
}

impl LoginForm {
	pub fn new(ep: EpRef, document: &Document) -> LoginForm {
		LoginForm {
			on_new_user_cb: LoginForm::add_new_user_login_event_listener(ep.clone(), document),
			on_forget_user_cb: LoginForm::add_forget_user_event_listener(ep.clone(), document),
			on_existing_user_cb: LoginForm::add_existing_user_login_event_listener(ep.clone(), document),
			on_local_game_cb: LoginForm::add_local_game_event_listener(ep.clone(), document),
			on_net_game_cb: LoginForm::add_net_game_event_listener(ep, document),
		}
	}

	pub fn show_state(&self, login: &LoginInfo, state: &LoginState) {
		let window = web_sys::window().expect("no global `window` exists");
		let document = window.document().expect("window has not document");

		dom::show_element_by_id(&document, "login_user_section");
		console::log!("Show state {:?}", state);

		match state {
			LoginState::NewUser => {
				dom::show_element_by_id(&document, "new_user_section");
				dom::hide_element_by_id(&document, "existing_user_section");
				dom::hide_element_by_id(&document, "choose_conn_section");
				dom::hide_element_by_id(&document, "opponent_section");
			}
			LoginState::ExistingUser => {
				dom::hide_element_by_id(&document, "new_user_section");
				dom::show_element_by_id(&document, "existing_user_section");
				dom::hide_element_by_id(&document, "choose_conn_section");
				dom::hide_element_by_id(&document, "opponent_section");
				match login {
					LoginInfo::NewUser { username } | LoginInfo::ExistingUser { username, .. } => {
						dom::set_inner_text_by_class(&document, "user_name", username.as_str());
						dom::set_field_value_by_id(
							&document,
							"invalidate_existing_user_btn",
							format!("Je ne suis pas {}", username).as_str(),
						);
					}
					_ => panic!("No user defined while going to step ExistingUser"),
				};
			}
			LoginState::ChooseGameMode => {
				dom::hide_element_by_id(&document, "new_user_section");
				dom::hide_element_by_id(&document, "existing_user_section");
				dom::show_element_by_id(&document, "choose_conn_section");
				dom::hide_element_by_id(&document, "opponent_section");
			}
			LoginState::OpponentName => {
				dom::hide_element_by_id(&document, "new_user_section");
				dom::hide_element_by_id(&document, "existing_user_section");
				dom::hide_element_by_id(&document, "choose_conn_section");
				dom::show_element_by_id(&document, "opponent_section");
			}
		}
	}

	pub fn hide(document: &Document) {
		dom::hide_element_by_id(document, "login_user_section");
	}

	fn clean_callbacks(&mut self) {
		if let Some(window) = web_sys::window() {
			if let Some(document) = window.document() {
				dom::by_id!(document, "new_user_form", HtmlFormElement).set_onsubmit(None);
				dom::by_id!(document, "existing_user_form", HtmlFormElement).set_onreset(None);
				dom::by_id!(document, "existing_user_form", HtmlFormElement).set_onsubmit(None);
				dom::by_id!(document, "local_game_btn", HtmlElement).set_onclick(None);
				dom::by_id!(document, "online_game_btn", HtmlElement).set_onclick(None);
			}
		}
	}

	fn add_new_user_login_event_listener(ep: EpRef, document: &Document) -> Closure<dyn FnMut(Event)> {
		let handler = move |event: Event| {
			event.prevent_default();
			let window = web_sys::window().expect("no global `window` exists");
			let document = window.document().expect("window has not document");
			let login_info = match dom::read_field_by_id(&document, "login") {
				username if !username.is_empty() => LoginInfo::NewUser { username },
				_ => {
					return;
				}
			};
			ep.set_login_info(login_info);
			ep.set_state(AppState::Login(LoginState::ChooseGameMode));
		};
		let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(Event)>);
		dom::by_id!(document, "new_user_form", HtmlFormElement).set_onsubmit(Some(closure.as_ref().unchecked_ref()));
		closure
	}

	fn add_forget_user_event_listener(ep: EpRef, document: &Document) -> Closure<dyn FnMut(Event)> {
		let handler = move |event: Event| {
			event.prevent_default();
			ep.set_state(AppState::Login(LoginState::NewUser));
		};
		let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(Event)>);
		dom::by_id!(document, "existing_user_form", HtmlFormElement)
			.set_onreset(Some(closure.as_ref().unchecked_ref()));
		closure
	}

	fn add_existing_user_login_event_listener(ep: EpRef, document: &Document) -> Closure<dyn FnMut(Event)> {
		let handler = move |event: web_sys::Event| {
			event.prevent_default();
			ep.set_state(AppState::Login(LoginState::ChooseGameMode));
		};
		let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(Event)>);
		dom::by_id!(document, "existing_user_form", HtmlFormElement)
			.set_onsubmit(Some(closure.as_ref().unchecked_ref()));
		closure
	}

	fn add_local_game_event_listener(ep: EpRef, document: &Document) -> Closure<dyn FnMut()> {
		let handler = move || {
			ep.set_state(AppState::Login(LoginState::OpponentName));
		};
		let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut()>);
		dom::by_id!(document, "local_game_btn", HtmlElement).set_onclick(Some(closure.as_ref().unchecked_ref()));
		closure
	}

	fn add_net_game_event_listener(ep: EpRef, document: &Document) -> Closure<dyn FnMut()> {
		let handler = move || {
			ep.set_state(AppState::Lobby);
		};
		let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut()>);
		dom::by_id!(document, "online_game_btn", HtmlElement).set_onclick(Some(closure.as_ref().unchecked_ref()));
		closure
	}
}

impl Drop for LoginForm {
	fn drop(&mut self) {
		self.clean_callbacks();
	}
}
