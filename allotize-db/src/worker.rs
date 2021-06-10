use wasm_bindgen::JsValue;
use web_sys::{MessageEvent, ServiceWorkerContainer};
use wasm_bindgen::prelude::*;
use crate::wasm_bindgen::JsCast;

#[wasm_bindgen(
    inline_js = "export function sleep(s) { return new Promise(r => setTimeout(r, s)); }"
)]
extern "C" {
    pub fn sleep(s: u32) -> js_sys::Promise;
}

#[derive(Debug)]
pub struct Worker {
    inner: ServiceWorkerContainer,
}

impl Worker {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();

        Self {
            inner: window.navigator().service_worker(),
        }
    }

    pub async fn send_message(&self, msg: JsValue) {
        let window = web_sys::window().unwrap();
        for i in 0..5 {
            if let Some(controller) = window.navigator().service_worker().controller() {
                controller
                    .post_message(&msg)
                    .expect("failed to post message");
                return;
            }
            wasm_bindgen_futures::JsFuture::from(
                unsafe{sleep(50 + 2u32.pow(i))}
            ).await.expect("sleep failed");
        }
    }

    pub fn controller(&self) -> Option<web_sys::ServiceWorker> {
        self.inner.controller()
    }

    pub fn register(&self) {
        let promise = self.inner.register("worker.js");
        wasm_bindgen_futures::spawn_local(async move {
            wasm_bindgen_futures::JsFuture::from(promise).await;
        });
    }
}
