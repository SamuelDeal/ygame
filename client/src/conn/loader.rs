use crate::utils::dom;
use web_sys::Document;

pub fn show_connecting(document: &Document) {
	dom::set_inner_text_by_id(document, "connection_label", "Connecting");
	dom::show_element_by_id(document, "connection_section");
	dom::show_element_by_id(document, "connection_spinner");
}

pub fn show_loading(document: &Document) {
	dom::set_inner_text_by_id(document, "connection_label", "Loading");
	dom::show_element_by_id(document, "connection_section");
	dom::show_element_by_id(document, "connection_spinner");
}

pub fn hide(document: &Document) {
	dom::hide_element_by_id(document, "connection_section");
}

pub fn show_duration(document: &Document, duration: &str) {
	dom::set_inner_text_by_id(
		document,
		"connection_label",
		format!("Reconnection dans {}", duration).as_str(),
	);
	dom::show_element_by_id(document, "connection_section");
	dom::hide_element_by_id(document, "connection_spinner");
}
