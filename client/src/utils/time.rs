use crate::utils::browser;
use std::time::Duration;

use wasm_bindgen::prelude::*;

pub struct Interval {
	id: JsIntervalId,
	callback: Closure<dyn FnMut()>,
}

impl Interval {
	pub fn new(callback: Closure<dyn FnMut()>, time: Duration) -> Self {
		let time_millis = time.as_millis();
		let time_millis = if time_millis > u32::MAX as u128 {
			u32::MAX
		} else {
			time_millis as u32
		};

		Interval {
			id: set_interval(&callback, time_millis),
			callback,
		}
	}

	fn clean_callback(&mut self) {
		clear_interval(self.id);
	}
}

impl Drop for Interval {
	fn drop(&mut self) {
		self.clean_callback();
	}
}

#[derive(Clone, Copy)]
struct JsIntervalId(i32);

#[cfg(target_arch = "wasm32")]
fn set_interval(closure: &Closure<dyn FnMut()>, time_millis: u32) -> JsIntervalId {
	JsIntervalId(browser::set_interval(closure, time_millis))
}

#[cfg(target_arch = "wasm32")]
fn clear_interval(interval_id: JsIntervalId) {
	browser::clear_interval(interval_id.0)
}

//Alternate version to silent the rust-analyser
//FIXME: you could find another way to handle rust-analyser
#[cfg(not(target_arch = "wasm32"))]
fn set_interval(closure: &Closure<dyn FnMut()>, time_millis: u32) -> JsIntervalId {
	#[allow(unused_unsafe)]
	unsafe {
		JsIntervalId(browser::set_interval(closure, time_millis))
	}
}

#[cfg(not(target_arch = "wasm32"))]
fn clear_interval(interval_id: JsIntervalId) {
	#[allow(unused_unsafe)]
	unsafe {
		browser::clear_interval(interval_id.0)
	}
}
