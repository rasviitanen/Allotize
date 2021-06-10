//! Test suite for the Web and headless browsers.
#![allow(unused_must_use)]
#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use allotize::js_util::sleep;
use allotize::TimedEvent;

#[wasm_bindgen_test]
async fn create_simple() {
    let millis = 250;
    {
        let callback = Closure::wrap(Box::new(move || console_log!("A")) as Box<dyn FnMut()>);
        let _sender = TimedEvent::new(callback, millis);
        wasm_bindgen_futures::JsFuture::from(sleep(millis * 4)).await;
    }
    {
        let callback = Closure::wrap(Box::new(move || console_log!("B")) as Box<dyn FnMut()>);
        let _sender = TimedEvent::new(callback, millis);
        wasm_bindgen_futures::JsFuture::from(sleep(millis * 4)).await;
    }
}
