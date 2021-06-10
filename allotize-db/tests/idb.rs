//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use allotize_db::{IdbFolder, IdbOpenDbRequest};
use std::path::Path;

wasm_bindgen_test_configure!(run_in_browser);

// Simple case of put and get
#[wasm_bindgen_test]
async fn store_put_and_get() {
    let idb = IdbOpenDbRequest::new("hadal-db")
        .open_with_store("hadal-store")
        .await
        .unwrap();
    idb.put("key1", "value1").await.unwrap();
    let res = idb.get("key1").await.unwrap();
    assert_eq!(res, JsValue::from_str("value1"));
}

// Simple case of put and get
#[wasm_bindgen_test]
async fn no_await_store_put_and_get() {
    let idb = IdbOpenDbRequest::new("hadal-db")
        .open_with_store("hadal-store")
        .await
        .unwrap();
    let mut actions = Vec::new();
    for _ in 1..100 {
        actions.push(idb.put("key1", "value1"));
    }
    for action in actions {
        action.await.unwrap();
    }
    let res = idb.get("key1").await.unwrap();
    assert_eq!(res, JsValue::from_str("value1"));
}

// Get non-existing value
#[wasm_bindgen_test]
async fn store_get_non_existing() {
    let idb = IdbOpenDbRequest::new("hadal-db")
        .open_with_store("hadal-store")
        .await
        .unwrap();
    let res = idb.get("non-existing-key").await;
    assert!(res == Ok(JsValue::UNDEFINED));
}

// Makes sure that we can open and close the database, and still acccess the
// internal values
#[wasm_bindgen_test]
async fn store_persistance() {
    let idb = IdbOpenDbRequest::new("hadal-db")
        .open_with_store("hadal-store")
        .await
        .unwrap();
    idb.put("key1", "value1").await.unwrap();

    drop(idb);

    let idb = IdbOpenDbRequest::new("hadal-db")
        .open_with_store("hadal-store")
        .await
        .unwrap();
    let res = idb.get("key1").await.unwrap();
    assert_eq!(res, JsValue::from_str("value1"));
}

// Makes sure that we can open and close the database, and still acccess the
// internal values
#[wasm_bindgen_test]
async fn store_multiple_open_persistance() {
    for i in 0..10 {
        {
            let idb = IdbOpenDbRequest::new("hadal-db")
                .open_with_store("hadal-store")
                .await
                .unwrap();
            for j in 0..100 {
                let key = format!("key{}", i);
                let value = format!("value{}", j);
                idb.put(&key, &value).await.unwrap();
                let res = idb.get(&key).await.unwrap();
                assert_eq!(res, JsValue::from_str(&value));
            }
        }
        // Make sure all open made 100 writes
        {
            let idb = IdbOpenDbRequest::new("hadal-db")
                .open_with_store("hadal-store")
                .await
                .unwrap();

            let key = format!("key{}", i);
            let res = idb.get(&key).await.unwrap();
            assert_eq!(res, JsValue::from_str("value99"));
        }
    }
}

// Makes sure that we can open and close the database, and still acccess the
// internal values
#[wasm_bindgen_test]
async fn file() {
    use std::io::Read;
    use std::io::Write;
    use std::io::{Seek, SeekFrom};

    let path = Path::new("/tmp/idb_file");
    let mut folder = IdbFolder::open(&path).await.unwrap();
    let mut file = folder.open_file(&path).await.unwrap();
    let key = "hello".as_bytes();
    file.write(key).unwrap();

    let mut buffer: Vec<u8> = vec![0; key.len()];

    file.seek(SeekFrom::Start(0)).unwrap();
    file.read(&mut buffer).unwrap();
    assert_eq!(std::str::from_utf8(&buffer).unwrap(), "hello");
}
