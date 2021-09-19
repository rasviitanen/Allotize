//! Test suite for the Web and headless browsers.

#![allow(unused_must_use)]
#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use allotize::{App, RtcMessage, VersionedComponent};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

async fn channel() -> (App, App) {
    let mut remote_app = App::new("remote".into(), false).await;
    let mut local_app = App::new("local".into(), true).await;

    // // Wait for the two pools to be connected
    local_app.require_channels(1).await;
    remote_app.require_channels(1).await;

    (local_app, remote_app)
}

#[wasm_bindgen_test]
async fn share_kv_pair() {
    // Configure two pools
    let (mut local_app, mut remote_app) = channel().await;

    // The real testing part
    remote_app
        .put_shared(
            "key1".into(),
            VersionedComponent::new_with_value("hello".into()),
        )
        .await;

    // The remote_app tells the local_app that all transactions are done
    let mut done_listener = local_app.txn().await.recv_cmd("done");
    let message = RtcMessage {
        command: RtcCommand::Done,
        key: "*".into(),
        value: None,
    };
    remote_app.txn().await.broadcast(&message).await;
    done_listener.result().await;

    let remote_component: VersionedComponent = remote_app.get_shared("key1").await.unwrap();
    let local_component: VersionedComponent = local_app.get_shared("key1").await.unwrap();
    let expect_value: wasm_bindgen::JsValue = "hello".into();

    // Check that the value is present for both apps
    assert_eq!(remote_component.data.v, expect_value,);
    assert_eq!(local_component.data.v, expect_value,);
}
