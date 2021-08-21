use std::fmt::{self, Display};

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
	Init,
	Move,
	Finished,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum UserRole {
	Seat1,
	Seat2,
	Observer,
}

impl Display for UserRole {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			UserRole::Seat1 => write!(f, "Player 1"),
			UserRole::Seat2 => write!(f, "Player 2"),
			UserRole::Observer => write!(f, "Observer"),
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
