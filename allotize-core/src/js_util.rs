use wasm_bindgen::prelude::*;

#[wasm_bindgen(
    inline_js = "export function sleep(s) { return new Promise(r => setTimeout(r, s)); }"
)]
extern "C" {
    pub fn sleep(s: u32) -> js_sys::Promise;
}
