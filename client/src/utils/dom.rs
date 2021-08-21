use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, HtmlInputElement};

#[macro_export]
macro_rules! by_id {
	($doc_var:expr, $id:expr, $type:ty) => {
		$doc_var
			.get_element_by_id($id)
			.expect(format!("#{} in the page", $id).as_str())
			.dyn_ref::<$type>()
			.expect(format!("#{} be an `stringify!($type)`", $id).as_str())
	};
}
pub(crate) use by_id;

pub fn show_element_by_id(document: &Document, id: &str) {
	by_id!(document, id, HtmlElement).set_hidden(false);
}

pub fn hide_element_by_id(document: &Document, id: &str) {
	by_id!(document, id, HtmlElement).set_hidden(true);
}

pub fn try_show_element_by_id(document: &Document, id: &str) {
	if let Some(element) = document.get_element_by_id(id) {
		if let Some(element) = element.dyn_ref::<HtmlElement>() {
			element.set_hidden(false);
		}
	}
}

pub fn try_hide_element_by_id(document: &Document, id: &str) {
	if let Some(element) = document.get_element_by_id(id) {
		if let Some(element) = element.dyn_ref::<HtmlElement>() {
			element.set_hidden(true);
		}
	}
}

pub fn try_set_inner_text_by_id(document: &Document, id: &str, content: &str) {
	if let Some(element) = document.get_element_by_id(id) {
		if let Some(element) = element.dyn_ref::<HtmlElement>() {
			element.set_inner_text(content);
		}
	}
}

pub fn read_field_by_id(document: &Document, id: &str) -> String {
	by_id!(document, id, HtmlInputElement).value()
}

pub fn set_inner_text_by_class(document: &Document, classname: &str, content: &str) {
	let elements = document.get_elements_by_class_name(classname);
	for i in 0..elements.length() {
		if let Some(element) = elements.item(i) {
			if let Some(element) = element.dyn_ref::<HtmlElement>() {
				element.set_inner_text(content);
			}
		}
	}
}

pub fn set_inner_text_by_id(document: &Document, id: &str, content: &str) {
	by_id!(document, id, HtmlElement).set_inner_text(content);
}

pub fn set_field_value_by_id(document: &Document, id: &str, content: &str) {
	by_id!(document, id, HtmlInputElement).set_value(content);
}
