use crate::utils::browser;
use crate::utils::dom;

#[cfg(debug_assertions)]
use crate::utils::console;

use std::fmt::{Display, Formatter, Result};
use std::panic;
use wasm_bindgen::JsValue;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Error {
	ProtocolError {
		details: String,
		should_reload: bool,
		source: Option<Box<dyn std::error::Error>>,
	},
	ConnectionError {
		details: String,
		source: Option<Box<dyn std::error::Error>>,
	},
	ImplError {
		details: String,
		source: Option<Box<dyn std::error::Error>>,
	},
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			Error::ProtocolError {
				details: _,
				should_reload,
				..
			} => {
				if *should_reload {
					write!(
						f,
						"Communicaton with the server failed. Please reload the page cleaning your cache"
					)
				} else {
					write!(f, "Communicaton with the server failed")
				}
			}
			Error::ConnectionError { details: _, source: _ } => write!(f, "Communicaton with the server failed"),
			Error::ImplError { details: _, source: _ } => write!(f, "Fatal internal error"),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match &self {
			Error::ProtocolError {
				details: _,
				should_reload: _,
				source,
			} => source.as_deref(),
			Error::ConnectionError { details: _, source } => source.as_deref(),
			Error::ImplError { details: _, source } => source.as_deref(),
		}
	}
}

#[derive(Debug)]
pub struct JsException {
	pub value: JsValue,
}

impl From<JsValue> for JsException {
	fn from(val: JsValue) -> Self {
		JsException { value: val }
	}
}

impl Display for JsException {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		write!(f, "{:?}", self.value)
	}
}

impl std::error::Error for JsException {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		None
	}
}

#[cfg(debug_assertions)]
pub fn render_error(e: &Error) {
	render_error_text(format!("{}", e).as_str());
	let (details, source) = match e {
		Error::ProtocolError {
			details,
			should_reload: _,
			source,
		} => (details, source),
		Error::ConnectionError { details, source } => (details, source),
		Error::ImplError { details, source } => (details, source),
	};
	console::error!("{}{}", details, render_error_source(source.as_deref()));
}

#[cfg(debug_assertions)]
fn render_error_source(e: Option<&dyn std::error::Error>) -> String {
	match e {
		Some(e) => format!("\n  caused by: {}", e) + render_error_source(e.source()).as_str(),
		None => "".to_owned(),
	}
}

#[cfg(not(debug_assertions))]
pub fn render_error(e: &Error) {
	render_error_text(format!("{}", e).as_str());
}

pub fn render_error_text(txt: &str) {
	let window = match web_sys::window() {
		Some(v) => v,
		None => {
			return;
		}
	};
	let document = match window.document() {
		Some(v) => v,
		None => {
			return;
		}
	};
	dom::try_show_element_by_id(&document, "error_section");
	dom::try_set_inner_text_by_id(&document, "error_div", txt);
	dom::try_hide_element_by_id(&document, "connection_section");
	dom::try_hide_element_by_id(&document, "login_user_section");
	dom::try_hide_element_by_id(&document, "lobby_section");
	dom::try_hide_element_by_id(&document, "game_section");
}

#[cfg(target_arch = "wasm32")]
fn clean_app() {
	browser::clean_app()
}

//Alternate version to silent the rust-analyser
//FIXME: you could find another way to handle rust-analyser
#[cfg(not(target_arch = "wasm32"))]
fn clean_app() {
	#[allow(unused_unsafe)]
	unsafe {
		browser::clean_app();
	}
}

pub fn panic_hook(panic_info: &panic::PanicInfo) {
	browser::clean_app();
	if cfg!(feature = "console_error_panic_hook") {
		console_error_panic_hook::hook(panic_info);
	}
	let error_text: String = match panic_info.payload().downcast_ref::<&str>() {
		Some(error_str) => format!("Erreur fatale: {}", error_str),
		_ => "Une erreur fatale est survenue".to_owned(),
	};
	render_error_text(error_text.as_str());
}
