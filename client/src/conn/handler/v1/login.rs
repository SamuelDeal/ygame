use crate::app::login::LoginInfo;
use crate::app::Application;
use crate::conn::handler::v1::running::RunningHandler;
use crate::conn::handler::MsgHandler;
use crate::conn::handler::NextHandler;
use crate::conn::socket::Socket;
use crate::err::Error;
use crate::err::JsException;
use crate::utils::console;
use crate::Result;
use ygame_protocol::v1 as msg;

use js_sys::ArrayBuffer;

#[derive(Debug)]
pub struct LoginHandler {}

impl MsgHandler for LoginHandler {
	fn on_start(&self, ws: &Socket, app: &mut Application) -> Result<()> {
		let (username, user_uid, session_uid) = match app.login_info() {
			LoginInfo::None => {
				return Err(Error::ImplError {
					details: "Login withou login info".to_owned(),
					source: None,
				});
			}
			LoginInfo::NewUser { username } => (username, None, None),
			LoginInfo::ExistingUser {
				username,
				user_uid,
				session_uid,
			} => {
				console::log!(
					"existing user found: user_uid = {:?}, session_uid = {:?}",
					user_uid,
					session_uid
				);
				(username, user_uid, session_uid)
			}
		};
		let login_msg = msg::LoginMessage {
			name: username,
			uid: user_uid,
			session_uid,
		};
		let login_msg = match rmp_serde::encode::to_vec(&login_msg) {
			Ok(login_msg) => login_msg,
			Err(e) => {
				return Err(Error::ImplError {
					details: "Serialisation failed".to_owned(),
					source: Some(Box::new(e)),
				});
			}
		};
		let login_msg_ref: &[u8] = &login_msg;
		match ws.send_buffer(login_msg_ref) {
			Err(e) => Err(Error::ConnectionError {
				details: "Unable to send message to the server".to_owned(),
				source: Some(Box::new(JsException::from(e))),
			}),
			Ok(_) => Ok(()),
		}
	}

	fn on_array_buf(&self, _ws: &Socket, app: &mut Application, buf: &ArrayBuffer) -> Result<NextHandler> {
		console::log!("received: {:?}", buf);
		let array = js_sys::Uint8Array::new(buf);
		let resp: std::result::Result<msg::LoginResponseMessage, rmp_serde::decode::Error> =
			rmp_serde::from_read_ref(&array.to_vec());
		match resp {
			Ok(msg::LoginResponseMessage {
				name,
				user_uid,
				session_uid,
			}) => {
				console::log!("Login response: user_uid = {}, session_uid = {}", user_uid, session_uid);
				app.set_login_info(LoginInfo::ExistingUser {
					username: name,
					session_uid: Some(session_uid),
					user_uid: Some(user_uid),
				});
				app.on_connected();
				Ok(Some(Box::new(RunningHandler::new())))
			}
			Err(e) => Err(Error::ProtocolError {
				details: "Desserialisation error".to_owned(),
				should_reload: false,
				source: Some(Box::new(e)),
			}),
		}
	}
}
