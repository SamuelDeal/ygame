mod handler;
pub mod loader;
mod socket;

use crate::app::Application;
use crate::conn::handler::NextHandler;
use crate::conn::socket::Socket;
use crate::ep::EpRef;
use crate::ep::SocketEntryPoint;
use crate::err;
use crate::utils::console;
use crate::utils::time;
use crate::Result;
use ygame_protocol as msg;

use std::cell::Cell;
use std::cell::RefCell;
use std::time::Duration;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::ErrorEvent;
use web_sys::{Document, MessageEvent};

/// Object representing the websocket connection to the server.
///
/// It manage the protocol implementation using vtable based polymorphysm (because I wanted to try it)
/// It also manage connection lost and auto-reconnection
///
/// It implements interior mutability for ease of use
pub struct Connection {
	entry_point: EpRef,
	server_address: String,
	handler: RefCell<Box<dyn handler::MsgHandler>>,
	websocket: RefCell<Option<Socket>>,
	reconnect_interval: Cell<Option<time::Interval>>,
	reconnect_wait_time: Cell<Duration>,
}

impl Drop for Connection {
	fn drop(&mut self) {
		console::error!("Connection cleaned !");
	}
}

impl Connection {
	pub fn new(
		document: &Document,
		entry_point: EpRef,
		server_override: Option<String>,
		subfolder: Option<String>,
	) -> Self {
		Connection {
			entry_point,
			server_address: Connection::get_ws_address(document, server_override, subfolder),
			handler: RefCell::new(Box::new(handler::handshake::HandshakeHandler {})),
			websocket: RefCell::new(None),
			reconnect_interval: Cell::new(None),
			reconnect_wait_time: Cell::new(Duration::from_millis(5000)),
		}
	}

	pub fn connect(&self, document: &Document) {
		self.disconnect();
		loader::show_connecting(document);
		let server_address = self.server_address.clone();
		self.set_socket(Socket::new(self.entry_point.clone(), server_address));
		self.reconnect_interval.replace(None);
	}

	pub fn disconnect(&self) {
		if let Some(ws) = self.websocket.borrow().as_ref() {
			let msg = msg::handshake::DisconnectMessage::FromClient;
			if let Ok(msg) = serde_json::to_string(&msg) {
				ws.send_text(msg.as_str()).ok();
			}
		}
		self.websocket.replace(None);
	}

	pub fn on_socket_error(&self, e: ErrorEvent) {
		console::log!("error event: {:?}", e.error());
		self.on_connection_lost();
	}

	pub fn on_reconnect_delay(&self, document: &Document) {
		self.connect(document)
	}

	pub fn on_message(&self, e: MessageEvent, app: &mut Application) {
		match self.handle_message(e, app) {
			Ok(None) => {}
			Ok(Some(new_handler)) => {
				self.set_handler(new_handler, app);
			}
			Err(e) => {
				err::render_error(&e);
			}
		}
	}

	pub fn on_connected(&self, app: &mut Application) {
		self.reconnect_wait_time.replace(Duration::from_millis(5000));
		self.set_handler(Box::new(handler::handshake::HandshakeHandler {}), app);
	}

	pub fn on_connection_lost(&self) {
		self.websocket.replace(None);
		let reconnect_wait_time = std::cmp::min(
			self.reconnect_wait_time.get() + Duration::new(1, 0),
			Duration::new(29, 0),
		);
		self.reconnect_wait_time.replace(reconnect_wait_time);
		let ep = self.entry_point.clone();

		let window = web_sys::window().expect("should have a window in this context");
		let document = window.document().expect("should have a document in this context");
		let performance = window.performance().expect("performance should be available");

		let start = performance.now();
		let cb = move || {
			let window = web_sys::window().expect("should have a window in this context");
			let document = window.document().expect("should have a document in this context");
			let elapsed = Duration::from_millis((performance.now() - start) as u64);
			if reconnect_wait_time <= elapsed {
				ep.on_websocket_reconnect_delay(&document);
			} else {
				let time_left = reconnect_wait_time - elapsed;
				loader::show_duration(&document, ((time_left.as_secs() + 1).to_string() + "s").as_str());
			}
		};
		let cb = Closure::wrap(Box::new(cb) as Box<dyn FnMut()>);
		self.reconnect_interval
			.replace(Some(time::Interval::new(cb, Duration::new(1, 0))));
		loader::show_duration(&document, (reconnect_wait_time.as_secs().to_string() + "s").as_str());
	}

	fn reconnect_wait_time(&self) -> Duration {
		self.reconnect_wait_time.get()
	}

	fn set_socket(&self, socket: Socket) {
		self.websocket.replace(Some(socket));
	}

	fn set_handler(&self, new_handler: Box<dyn handler::MsgHandler>, app: &mut Application) {
		console::error!("New handler: {:?}", new_handler.as_ref());
		self.handler.replace(new_handler);
		if let Some(ws) = self.websocket.borrow().as_ref() {
			match self.handler.borrow().on_start(ws, app) {
				Ok(()) => {}
				Err(e) => {
					err::render_error(&e);
				}
			}
		} else {
			panic!("Connected but without socket ?!!");
		}
	}

	pub fn create_game(&self) {
		if let Some(ws) = self.websocket.borrow().as_ref() {
			self.handler.borrow_mut().create_game(ws);
		} else {
			panic!("No websocket available")
		}
	}

	fn handle_message(&self, e: MessageEvent, app: &mut Application) -> Result<NextHandler> {
		if let Some(ws) = self.websocket.borrow().as_ref() {
			if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
				self.handler.borrow().on_array_buf(&ws, app, &abuf)
			} else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
				self.handler.borrow().on_blob(&ws, app, &blob)
			} else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
				console::log!("txt message received: {}", String::from(&txt));
				if txt == "ping" {
					match ws.send_text("pong") {
						Ok(_) => Ok(None),
						Err(e) => Err(err::Error::ConnectionError {
							details: "Unable to send message tp the server".to_owned(),
							source: Some(Box::new(err::JsException::from(e))),
						}),
					}
				} else if txt == "pong" {
					Ok(None)
				} else {
					let resp: serde_json::Result<msg::error::ErrorMessage> = serde_json::from_str(&String::from(&txt));
					match resp {
						Ok(resp) => {
							console::error!("{:?}", resp);
							Err(err::Error::ProtocolError {
								details: resp.error_description,
								should_reload: resp.should_reload,
								source: None,
							})
						}
						Err(_) => self.handler.borrow().on_text(&ws, app, String::from(txt).as_str()),
					}
				}
			} else {
				Err(err::Error::ConnectionError {
					details: format!("message event, received Unknown: {:?}", e.data()),
					source: None,
				})
			}
		} else {
			Err(err::Error::ImplError {
				details: "No opened socket but we received something anyway ?!!".to_owned(),
				source: None,
			})
		}
	}

	fn get_ws_address(document: &Document, server_override: Option<String>, subfolder: Option<String>) -> String {
		//Get current page protocol and domain
		let location = document.location().expect("document has no location");
		let is_https = location.protocol().expect("no protocol") == "https:";

		let mut protocol = if is_https { "wss://" } else { "ws://" };
		let mut domain: String = location.host().expect("no host for url location");
		let mut port: String = String::from("");
		let mut subfolder: String = subfolder.unwrap_or("".to_owned()).trim_start_matches("/").to_owned();

		if let Some(override_value) = server_override {
			if override_value.contains("://") {
				protocol = "";
				domain = override_value.trim_end_matches("/").to_owned();
				subfolder = "".to_owned();
			} else if override_value.parse::<u16>().is_ok() {
				domain = location
					.hostname()
					.expect("no hostname for url location")
					.trim_end_matches("/")
					.to_owned();
				port = format!(":{}", override_value);
			} else {
				domain = override_value.trim_end_matches("/").to_owned();
			}
		}
		format!("{}{}{}/{}", protocol, domain, port, subfolder)
	}
}
