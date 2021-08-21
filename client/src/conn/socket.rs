use crate::ep::AppEntryPoint;
use crate::ep::EpRef;
use crate::ep::SocketEntryPoint;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

pub struct Socket {
	websocket: WebSocket,
	on_error_cb: Closure<dyn FnMut(ErrorEvent)>,
	on_open_cb: Closure<dyn FnMut(JsValue)>,
	on_msg_cb: Closure<dyn FnMut(MessageEvent)>,
	on_close_cb: Closure<dyn FnMut(JsValue)>,
}

impl Socket {
	pub fn new(ep: EpRef, server_address: String) -> Socket {
		let url = format!("{}/websocket", server_address.trim_end_matches("/"));
		let ws = WebSocket::new(url.as_str()).expect("Unable to open a websocket");
		ws.set_binary_type(web_sys::BinaryType::Arraybuffer); // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling

		let ep_clone = ep.clone();
		let onerror_callback = move |e: ErrorEvent| {
			ep_clone.on_websocket_error(e);
		};
		let onerror_callback = Closure::wrap(Box::new(onerror_callback) as Box<dyn FnMut(ErrorEvent)>);
		ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));

		let ep_clone = ep.clone();
		let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
			ep_clone.on_websocket_message(e);
		}) as Box<dyn FnMut(MessageEvent)>);
		ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

		let ep_clone = ep.clone();
		let onopen_callback = move |_| {
			ep_clone.on_websocket_connected();
		};
		let onopen_callback = Closure::wrap(Box::new(onopen_callback) as Box<dyn FnMut(JsValue)>);
		ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));

		let ep_clone = ep.clone();
		let onclose_callback = move |_| {
			ep_clone.on_websocket_connection_lost();
		};
		let onclose_callback = Closure::wrap(Box::new(onclose_callback) as Box<dyn FnMut(JsValue)>);
		ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));

		Socket {
			websocket: ws,
			on_error_cb: onerror_callback,
			on_open_cb: onopen_callback,
			on_msg_cb: onmessage_callback,
			on_close_cb: onclose_callback,
		}
	}

	pub fn send_buffer(&self, buf: &[u8]) -> Result<(), JsValue> {
		self.websocket.send_with_u8_array(buf)
	}

	pub fn send_text(&self, txt: &str) -> Result<(), JsValue> {
		self.websocket.send_with_str(txt)
	}

	fn close(&self) {
		self.clean_callbacks();
		self.websocket.close().ok();
	}

	fn clean_callbacks(&self) {
		self.websocket.set_onerror(None);
		self.websocket.set_onmessage(None);
		self.websocket.set_onopen(None);
		self.websocket.set_onclose(None);
	}
}

impl Drop for Socket {
	fn drop(&mut self) {
		self.close();
	}
}
