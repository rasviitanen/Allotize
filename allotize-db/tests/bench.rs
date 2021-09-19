#![cfg(target_arch = "wasm32")]

extern crate getrandom;
extern crate wasm_bindgen_test;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use allotize_db::{IdbOpenDbRequest, KvStore};
use std::path::PathBuf;

use web_sys::console;

wasm_bindgen_test_configure!(run_in_browser);

// Should overwrite existent value
#[wasm_bindgen_test]
async fn store_1000() {
    let test_path = PathBuf::from("/tmp/bench");
    let mut store = KvStore::open(&test_path).await.unwrap();

    let n = 1000;
    let mut random = vec![1; n];
    getrandom::getrandom(&mut random).unwrap();

    console::time_with_label("store_1000");
    console::time_with_label("store_1000_setters");
    {
        // Set 1000
        for i in 0..n {
            let mut txn = store.txn();
            txn.set(format!("key{}", i), format!("value{}", random[i]))
                .await
                .unwrap();
        }
    }
    console::time_end_with_label("store_1000_setters");
    console::time_with_label("store_1000_getters");
    {
        // Get 1000
        for i in 0..n {
            let mut txn = store.txn();
            assert_eq!(
                txn.get(format!("key{}", i)).await.unwrap(),
                Some(format!("value{}", random[i]))
            );
        }
    }
    console::time_end_with_label("store_1000_getters");
    console::time_end_with_label("store_1000");
}

// #[wasm_bindgen_test]
// async fn idb_1000() {
//     let idb = IdbOpenDbRequest::new("allotize-db")
//         .open_with_store("allotize-store")
//         .await
//         .unwrap();

//     let n = 1000;
//     let mut random = vec![1; n];
//     getrandom::getrandom(&mut random).unwrap();

//     console::time_with_label("idb_1000");
//     console::time_with_label("idb_1000_setters");
//     {
//         // Set 1000
//         for i in 0..n {
//             idb.put(&format!("key{}", i), &format!("value{}", random[i]))
//                 .await
//                 .unwrap();
//         }
//     }
//     console::time_end_with_label("idb_1000_setters");
//     console::time_with_label("idb_1000_getters");
//     {
//         // Get 1000
//         for i in 0..n {
//             assert_eq!(
//                 idb.get(&format!("key{}", i)).await.unwrap(),
//                 JsValue::from_str(&format!("value{}", random[i]))
//             );
//         }
//     }
//     console::time_end_with_label("idb_1000_getters");
//     console::time_end_with_label("idb_1000");
// }
