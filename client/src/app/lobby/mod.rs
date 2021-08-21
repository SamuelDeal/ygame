pub mod form;

#[derive(Debug, Clone, PartialEq)]
pub enum GameStatus {
	Joinable = 1,
	Rejoinable = 2,
	Full = 3,
	Finished = 4,
}

#[derive(Debug, Clone)]
pub struct GameOverview {
	pub id: String,
	pub name: String,
	pub status: GameStatus,
}
