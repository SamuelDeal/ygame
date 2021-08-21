use web_sys::Window;

pub const HIST_GAME_PREFIX: &'static str = "ygame_";
pub const HIST_INIT_PREFIX: &'static str = "init_";

pub fn get_current_hist_state(window: &Window) -> Option<String> {
	let history = window.history().expect("window has no history");
	let state = history.state();
	if let Ok(state) = state {
		let state = state.as_string().unwrap_or_default();
		if state.is_empty() {
			None
		} else {
			Some(state)
		}
	} else {
		None
	}
}

pub fn is_first_game_state(state: &str) -> bool {
	if !is_game_state(state) {
		return false;
	}
	let state = &state[HIST_GAME_PREFIX.len()..];
	state.starts_with(HIST_INIT_PREFIX)
}

pub fn is_game_state(state: &str) -> bool {
	state.starts_with(HIST_GAME_PREFIX)
}
