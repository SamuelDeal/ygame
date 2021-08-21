#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

mod ep;
mod err;
mod utils;

#[path = "app/app.rs"]
mod app;

#[path = "conn/conn.rs"]
mod conn;

use crate::app::Application;
use crate::conn::Connection;
use crate::ep::JsEntryPoint;
use crate::utils::console;

use std::cell::RefCell;
use std::panic;
use std::rc::Rc;

extern crate console_error_panic_hook;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type Result<T> = std::result::Result<T, err::Error>;

#[wasm_bindgen(start)]
pub fn wasm_main() -> std::result::Result<(), JsValue> {
	panic::set_hook(Box::new(err::panic_hook));
	Ok(())
}

#[wasm_bindgen]
pub struct WebClient {
	app: Option<Rc<RefCell<Application>>>,
	conn: Option<Rc<Connection>>,
}

#[wasm_bindgen]
impl WebClient {
	#[wasm_bindgen(constructor)]
	pub fn new(server_override: JsValue, subfolder: JsValue) -> WebClient {
		let server_override = WebClient::parse_server_override(server_override);
		let subfolder = WebClient::parse_subfolder(subfolder);

		let window = web_sys::window().expect("no global `window` exists");
		let document = window.document().expect("window has no document");
		let entry_point = Rc::new(RefCell::new(JsEntryPoint::new()));

		let app = Rc::new(RefCell::new(Application::new(
			&window,
			&document,
			subfolder.clone(),
			entry_point.clone(),
		)));
		let conn = Rc::new(Connection::new(
			&document,
			entry_point.clone(),
			server_override,
			subfolder,
		));
		let result = WebClient {
			app: Some(app.clone()),
			conn: Some(conn.clone()),
		};
		entry_point.borrow_mut().set_app(&app);
		entry_point.borrow_mut().set_conn(&conn);

		result
	}

	pub fn start(&mut self) -> std::result::Result<(), JsValue> {
		if let Some(app) = &self.app {
			if let Some(conn) = &self.conn {
				app.borrow_mut().start(conn);
				Ok(())
			} else {
				Err(js_sys::Error::new("no connection").into())
			}
		} else {
			Err(js_sys::Error::new("no application").into())
		}
	}

	pub fn clean(&mut self) -> std::result::Result<(), JsValue> {
		self.app = None;
		self.conn = None;
		Ok(())
	}

	fn parse_server_override(server_override: JsValue) -> Option<String> {
		if server_override.is_function()
			|| server_override.is_object()
			|| server_override.is_null()
			|| server_override.is_symbol()
			|| server_override.is_undefined()
		{
			None
		} else if server_override.is_string() {
			server_override.as_string().filter(|v| !v.is_empty())
		} else {
			server_override
				.as_f64()
				.map(|v| v.to_string())
				.filter(|v| !v.is_empty())
		}
	}

	fn parse_subfolder(subfolder: JsValue) -> Option<String> {
		if subfolder.is_string() {
			subfolder
				.as_string()
				.filter(|v| !v.is_empty())
				.map(|v| if v.starts_with("/") { v } else { format!("/{}", v) })
		} else {
			None
		}
	}
}

impl Drop for WebClient {
	fn drop(&mut self) {
		console::error!("WebClient is dropped !");
	}
}
