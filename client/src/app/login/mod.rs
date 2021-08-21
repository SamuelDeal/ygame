pub mod form;

use web_sys::Window;

#[derive(Clone)]
pub enum LoginInfo {
	None,
	NewUser {
		username: String,
	},
	ExistingUser {
		username: String,
		user_uid: Option<String>,
		session_uid: Option<String>,
	},
}

impl LoginInfo {
	pub fn is_none(&self) -> bool {
		matches!(self, LoginInfo::None)
	}

	pub fn save(&self, window: &Window) {
		let storage = window.local_storage().unwrap().unwrap();
		match self {
			LoginInfo::None => {
				storage.remove_item("username").ok();
				storage.remove_item("user_uid").ok();
				storage.remove_item("session_uid").ok();
			}
			LoginInfo::NewUser { username } => {
				storage.set_item("username", username.as_str()).ok();
				storage.remove_item("user_uid").ok();
				storage.remove_item("session_uid").ok();
			}
			LoginInfo::ExistingUser {
				username,
				user_uid,
				session_uid,
			} => {
				storage.set_item("username", username.as_str()).ok();
				match user_uid {
					Some(user_uid) => {
						storage.set_item("user_uid", user_uid.as_str()).ok();
					}
					None => {
						storage.remove_item("user_uid").ok();
					}
				}
				match session_uid {
					Some(session_uid) => {
						storage.set_item("session_uid", session_uid.as_str()).ok();
					}
					None => {
						storage.remove_item("session_uid").ok();
					}
				}
			}
		}
	}

	pub fn load(window: &Window) -> LoginInfo {
		let storage = window.local_storage().unwrap().unwrap();
		let username = match storage.get_item("username") {
			Ok(Some(value)) if !value.is_empty() => value,
			_ => return LoginInfo::None,
		};
		let user_uid = match storage.get_item("user_uid") {
			Ok(Some(value)) if !value.is_empty() => Some(value),
			_ => None,
		};
		let session_uid = match storage.get_item("session_uid") {
			Ok(Some(value)) if !value.is_empty() => Some(value),
			_ => None,
		};
		if session_uid.is_none() && user_uid.is_none() {
			LoginInfo::NewUser { username }
		} else {
			LoginInfo::ExistingUser {
				username,
				user_uid,
				session_uid,
			}
		}
	}
}
