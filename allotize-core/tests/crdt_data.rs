//! Test suite for the Web and headless browsers.

#![allow(unused_must_use)]
#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use allotize_core::{js_util::sleep, App, VersionedComponent};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

async fn channel() -> (App, App) {
    let mut remote_app = App::new("remote".into(), false).await;
    let mut local_app = App::new("local".into(), true).await;

    // Wait for the two pools to be connected
    local_app.require_channels(1).await;
    remote_app.require_channels(1).await;

    (local_app, remote_app)
}

#[wasm_bindgen_test]
async fn remote_is_ahead() {
    let (mut local_app, mut remote_app) = channel().await;

    remote_app
        .crdt_put(
            remote_app.identity().username,
            "key1".into(),
            VersionedComponent::default(),
        )
        .await;

    wasm_bindgen_futures::JsFuture::from(sleep(1000)).await;

    assert_eq!(
        remote_app.get_shared("key1").await.unwrap().data.v,
        local_app.get_shared("key1").await.unwrap().data.v,
    );
}

#[wasm_bindgen_test]
async fn merge_conflict() {
    // Configure two pools
    let mut local_app = App::new("local".into(), false).await;
    let mut remote_app = App::new("remote".into(), true).await;

    // Create two unconnected changes on local
    assert!(local_app
        .crdt_put(
            local_app.identity().username,
            "key1".into(),
            VersionedComponent::new_with_value("value1".into()),
        )
        .await
        .is_ok());

    assert_eq!(
        local_app.crdt_get("key1").await.unwrap().data.v,
        wasm_bindgen::JsValue::from("key1"),
    );

    local_app
        .crdt_put(
            local_app.identity().username,
            "key1".into(),
            VersionedComponent::new_with_value("value2".into()),
        )
        .await;

    // Create an unconnected change on remote
    remote_app
        .crdt_put(
            remote_app.identity().username,
            "key1".into(),
            VersionedComponent::new_with_value("value3".into()),
        )
        .await;

    // Connect the channels
    remote_app.require_channels(1).await;

    // Create a change while connected
    remote_app
        .crdt_put(
            remote_app.identity().username,
            "key1".into(),
            VersionedComponent::new_with_value("value4".into()),
        )
        .await;

    wasm_bindgen_futures::JsFuture::from(sleep(200)).await;
    // Check that the value is present for both apps
    assert_eq!(
        remote_app.get_shared("key1").await.unwrap().data.v,
        local_app.get_shared("key1").await.unwrap().data.v,
    );

    let expect_value: wasm_bindgen::JsValue = "value2".into();
    assert_eq!(
        local_app.crdt_get("key1").await.unwrap().data.v,
        expect_value,
    );
}
