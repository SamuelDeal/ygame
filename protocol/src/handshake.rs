use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct HelloMessage {
	pub known_protocols: Vec<u32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum HelloResponseMessage {
	Success { protocol_version: u32 },
	Failure { should_reload: bool },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum DisconnectMessage {
	FromClient,
	FromServer,
}
