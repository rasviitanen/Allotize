//! Hadal is a framework for building web applications and components without servers.

//! The easiest way of describing Hadal, is essentially as a P2P database that uses
//! conflict free datatypes in order to keep all peers in sync.
//!
//! Hadal runs in a Virtual Machine inside your browser,
//! and uses WebRTC to communicate with other peers.
//! Data is stored in the browser using IndexedDB.

extern crate crdts;
extern crate js_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;

////////////////////////////////////////////////////////////////////////////////
//////////////                Macros and Statics                  //////////////
////////////////////////////////////////////////////////////////////////////////

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
        format!("\n  ├  {} {}", $fmt, get_the_args!($($args)*))
    };
}

#[allow(unused_macros)]
macro_rules! info {
    ($t:tt) => {
        format!("\n {}", $t)
    };
    ($cmd:expr, $($t:tt)*) => {
        // #[cfg(feature = "enable_logging")]
        crate::log(&format!(
        "[INFO] ({} → {}::{}) {}",
        $cmd,
        module_path!(),
        line!(),
        get_the_args!($($t)*)))
    };
}

#[wasm_bindgen]
extern "C" {
    /// Extern JS Binding for `console.log()`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

////////////////////////////////////////////////////////////////////////////////
//////////////                      Modules                       //////////////
////////////////////////////////////////////////////////////////////////////////

/// The app context
mod app;
/// The P2P pool, where peers share component information in real-time
mod com;
/// System to identify users and make sure they are allowed to read/write
/// component information
mod identity;
/// Traits
mod net_traits;
// JS utilities
pub mod js_util;

use wasm_bindgen::prelude::*;

// Internal prelude exports
pub use app::App;
pub use com::com_traits::RtcMessage;
pub use com::rtcpool::RtcPool;
pub use com::rtctransaction::RtcTxn;
pub use com::timed_event::TimedEvent;
pub use identity::Identity;
pub use net_traits::VersionedComponent;
