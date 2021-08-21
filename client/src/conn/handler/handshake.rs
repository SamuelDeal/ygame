use crate::app::Application;
use crate::conn::handler::v1::login::LoginHandler;
use crate::conn::handler::MsgHandler;
use crate::conn::handler::NextHandler;
use crate::conn::socket::Socket;
use crate::err;
use crate::err::JsException;
use crate::utils::console;
use crate::Result;
use ygame_protocol::handshake as msg;
use ygame_protocol::v1 as msg_v1;

#[derive(Debug)]
pub struct HandshakeHandler {}

impl MsgHandler for HandshakeHandler {
	fn on_start(&self, ws: &Socket, _app: &mut Application) -> Result<()> {
		let hello_msg = msg::HelloMessage {
			known_protocols: vec![1],
		};
		let hello_msg = match serde_json::to_string(&hello_msg) {
			Ok(hello_msg) => hello_msg,
			Err(e) => {
				return Err(err::Error::ImplError {
					details: "Serialisation failed".to_owned(),
					source: Some(Box::new(e)),
				});
			}
		};
		match ws.send_text(hello_msg.as_str()) {
			Ok(_) => Ok(()),
			Err(e) => Err(err::Error::ConnectionError {
				details: "Unable to send message tp the server".to_owned(),
				source: Some(Box::new(JsException::from(e))),
			}),
		}
	}

	fn on_text(&self, _socket: &Socket, _app: &mut Application, txt: &str) -> Result<NextHandler> {
		console::log!("received: {}", txt);
		let resp: serde_json::Result<msg::HelloResponseMessage> = serde_json::from_str(&String::from(txt));
		match resp {
			Ok(msg::HelloResponseMessage::Success { protocol_version }) => {
				if protocol_version == msg_v1::VERSION {
					Ok(Some(Box::new(LoginHandler {})))
				} else {
					Err(err::Error::ProtocolError {
						details: "No compatible protocol version".to_owned(),
						should_reload: false,
						source: None,
					})
				}
			}
			Ok(msg::HelloResponseMessage::Failure { should_reload }) => Err(err::Error::ProtocolError {
				details: "No compatible protocol version".to_owned(),
				should_reload,
				source: None,
			}),
			Err(e) => Err(err::Error::ProtocolError {
				details: "Desserialisation error".to_owned(),
				should_reload: false,
				source: Some(Box::new(e)),
			}),
		}
	}
}
