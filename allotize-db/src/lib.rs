// #![deny(missing_docs)]
//! A simple key/value store.

extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
#[macro_use]
extern crate serde_derive;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (crate::log(&format_args!($($t)*).to_string()))
}

#[allow(unused_macros)]
macro_rules! get_the_args {
    ($ fmt : expr) => {
        format!("\n  ╰  {}", $fmt)
    };
    ($ fmt : expr, $ ($ args : tt) *) => {
        format!("\n  ├  {} {}", $fmt, get_the_args!($($args)*));
    };
}

#[allow(unused_macros)]
macro_rules! info {
    ($t:tt) => {
        format!("\n {}", $t)
    };
    ($cmd:expr, $($t:tt)*) => {
        crate::log(&format!(
            "[INFO] ({} → {}::{}) {}",
            $cmd,
            module_path!(),
            line!(),
            get_the_args!($($t)*)));
    };
}

#[wasm_bindgen]
extern "C" {
    /// Extern JS Binding for `console.log()`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

mod engine;
mod error;
mod idb;
// mod worker;
// mod thread_pool;
// mod engines;

pub use engine::KvStore;
pub use error::{KvsError, Result};
pub use idb::{IdbFile, IdbFolder, IdbHandle, IdbOpenDbRequest};

use wasm_bindgen::prelude::*;
