use uuid::Uuid;

pub struct User {
	pub name: String,
	pub session_uid: Uuid,
	pub uid: Uuid,
}
