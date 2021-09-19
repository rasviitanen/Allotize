//! Test suite for the Web and headless browsers.
#![allow(unused_must_use)]
#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use allotize::{Identity, RtcCommand, RtcMessage, RtcPool, VersionedComponent};

#[wasm_bindgen_test]
async fn p2p_message() {
    // Configure two pools
    let identity_one = Identity::new("test-user-1");
    let mut pool_one = RtcPool::new("test-room", identity_one);

    let identity_two = Identity::new("test-user-2");
    let mut pool_two = RtcPool::new("test-room", identity_two);

    // Connect the two pools
    pool_one.setup(false); // Sends a connection request
    pool_two.setup(true);

    // Wait for pool two to establish a data channel
    pool_two.require_channels(1).await;

    // Set up a listener on the pool, that listens for 'Uh'
    // Broadcast a message 'Oh', and expect the receiving end to return an error
    let mut receiver = pool_one.txn().recv_cmd(RtcCommand::Put);
    let message = RtcMessage {
        command: RtcCommand::Put,
        key: "hello".into(),
        value: Some(VersionedComponent::new_with_value("hello".into())),
    };
    pool_two.txn().broadcast(&message).await;
    assert!(receiver.result().await.is_err());

    // Set up a listener on the pool, that listens fro 'OK'
    // Broadcast a message 'OK', and make sure the receiver accepted the message
    receiver = pool_one.txn().recv_cmd(RtcCommand::Put);
    pool_two.txn().broadcast(&message).await;
    assert!(receiver.result().await.is_ok());
}

// #[wasm_bindgen_test]
// async fn p2p_multi_message() {
//     // Configure two pools
//     let identity_one = Identity::new("test-user-1");
//     let mut pool_one = RtcPool::new("test-room", identity_one);

//     let identity_two = Identity::new("test-user-2");
//     let mut pool_two = RtcPool::new("test-room", identity_two);

//     let identity_three = Identity::new("test-user-3");
//     let mut pool_three = RtcPool::new("test-room", identity_three);

//     // Connect three pools
//     pool_one.setup(false);
//     pool_two.setup(false);
//     pool_three.setup(true);

//     pool_three.require_channels(2).await;

//     let message = RtcMessage {
//         command: "put".into(),
//         key: "hello".into(),
//         value: Some("hello".into()),
//     };

//     // Set up a listener on the pool, that listens for a `put` command
//     let mut recv_pool_one = pool_one.txn().recv_cmd("put");
//     let mut recv_pool_two = pool_two.txn().recv_cmd("put");

//     pool_three.txn().broadcast(&message).await;
//     assert!(recv_pool_one.result().await.is_ok());
//     assert!(recv_pool_two.result().await.is_ok());
// }

// #[wasm_bindgen_test]
// async fn reconnect() {
//     // Configure two pools
//     let identity_one = Identity::new("test-user-1");
//     let identity_two = Identity::new("test-user-2");

//     let mut pool_one = RtcPool::new("test-room", identity_one);
//     let mut pool_two = RtcPool::new("test-room", identity_two);

//     // Connect the two pools
//     pool_one.setup(false); // Sennds a connection request
//     pool_two.setup(true); // Wait for pool two to establish a connection

//     pool_one.require_channels(1).await;
//     pool_two.require_channels(1).await;

//     drop(pool_one);
//     drop(pool_two);

//     let identity_one = Identity::new("test-user-1");
//     let identity_two = Identity::new("test-user-2");

//     let mut pool_one = RtcPool::new("test-room", identity_one);
//     let mut pool_two = RtcPool::new("test-room", identity_two);

//     pool_one.setup(false); // Sennds a connection request
//     pool_two.setup(true); // Wait for pool two to establish a connection

//     pool_one.require_channels(1).await;
//     pool_two.require_channels(1).await;

//     // Set up a listener on the pool, that listens for 'Uh'
//     // Broadcast a message 'Oh', and expect the receiving end to return an error
//     let mut receiver = pool_one.txn().recv_cmd("get");
//     let message = RtcMessage {
//         command: "put".into(),
//         key: "key1".into(),
//         value: Some("value1".into()),
//     };
//     pool_two.txn().broadcast(&message).await;
//     assert!(receiver.result().await.is_err());

//     // Set up a listener on the pool, that listens fro 'OK'
//     // Broadcast a message 'OK', and make sure the receiver accepted the message
//     receiver = pool_one.txn().recv_cmd("put");
//     pool_two.txn().broadcast(&message).await;
//     assert!(receiver.result().await.is_ok());
// }
