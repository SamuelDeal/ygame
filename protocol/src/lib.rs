//Protocol definition here
//Each submodule follow i's own rules:
//
// * Handshake:
//   This module should never change.
//   Messages are designed to be sent in json text format. It is used to choose subsequent protocol
//
// * Error:
//   The main error message is designed to be sent in json text format by the server. It should also never change
//   However you could add new error code if desired.
//   Error codes are not meant to be changed, but if this happen, it should not be a fatal issue.
//
// * Versionned protocol:
//   While finalized you should not changed a protocol version.
//   You can add as many protocol as you want though
//   The application is designed to choose the highest protocol number during handshake

pub mod error;
pub mod handshake;
pub mod v1;

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
