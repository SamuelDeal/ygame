#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => ( crate::utils::browser::console_log(&format_args!($($t)*).to_string()))
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! error {
    ($($t:tt)*) => ( crate::utils::browser::console_error(&format_args!($($t)*).to_string()))
}

//Alternate version to silent the rust-analyser
//FIXME: you could find another way to handle rust-analyser
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (
        #[allow(unused_unsafe)]
        unsafe{
            crate::utils::browser::console_log(&format_args!($($t)*).to_string())
        }
    )
}
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! error {
    ($($t:tt)*) => (
        #[allow(unused_unsafe)]
        unsafe{
            crate::utils::browser::console_error(&format_args!($($t)*).to_string())
        }
    )
}

pub(crate) use error;
pub(crate) use log;
