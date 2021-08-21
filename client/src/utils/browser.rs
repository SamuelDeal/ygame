use wasm_bindgen::prelude::*;
use web_sys::Document;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_name = setInterval)]
	pub fn set_interval(closure: &Closure<dyn FnMut()>, millis: u32) -> i32;

	#[wasm_bindgen(js_name = clearInterval)]
	pub fn clear_interval(interval_id: i32);

	#[wasm_bindgen(js_namespace = console, js_name = log)]
	pub fn console_log(s: &str);

	#[wasm_bindgen(js_namespace = console, js_name = error)]
	pub fn console_error(s: &str);
}

const CLEAN_APP_JS: &str = r#"
try {
    if(window.webClient != null) {
        window.webClient.clean();
    }
}
catch(e) {
    console.error(e);
}
window.webClient = null;
"#;

#[cfg(target_arch = "wasm32")]
pub fn clean_app() {
	js_sys::eval(CLEAN_APP_JS).ok();
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_unsafe)]
pub fn clean_app() {
	unsafe {
		js_sys::eval(CLEAN_APP_JS).ok();
	}
}

pub fn get_url_path(document: &Document) -> Option<String> {
	if let Some(location) = document.location() {
		if let Ok(url) = location.pathname() {
			Some(url)
		} else {
			None
		}
	} else {
		None
	}
}

pub fn to_full_url(document: &Document, path: &str) -> String {
	let location = document.location().expect("No location for document");
	let origin = location.origin().expect("No origin for location");
	format!("{}/{}", origin.trim_end_matches("/"), path.trim_start_matches("/"))
}
