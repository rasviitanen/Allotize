//! Test suite for the Web and headless browsers.

#![feature(async_closure)]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use allotize_db::{IdbFolder, KvStore};
use std::path::{Path, PathBuf};

wasm_bindgen_test_configure!(run_in_browser);

// Should overwrite existent value
#[wasm_bindgen_test]
async fn overwrite_value() {
    let test_path = PathBuf::from("/tmp/1");
    let mut store = KvStore::open(&test_path).await.unwrap();

    store.txn().set("key13".to_owned(), "value1").await.unwrap();
    assert_eq!(
        store.txn().get("key13".to_owned()).await.unwrap(),
        Some("value1".to_owned())
    );
    store.txn().set("key13".to_owned(), "value2").await.unwrap();
    assert_eq!(
        store.txn().get("key13".to_owned()).await.unwrap(),
        Some("value2".to_owned())
    );
}

#[wasm_bindgen_test]
async fn read_old_values_with_active_substore() {
    let test_path = PathBuf::from("/tmp/2");
    let mut store = KvStore::open(&test_path).await.unwrap();

    store.txn().set("key1".to_owned(), "value1").await.unwrap();
    store.add_substore(Path::new("/component1")).await.unwrap();

    store
        .txn()
        .set_scoped(
            "key1".to_owned(),
            "value2".to_owned(),
            Some(Path::new("/component1")),
        )
        .await
        .unwrap();

    assert_eq!(
        store.txn().get("key1".to_owned()).await.unwrap(),
        Some("value1".to_owned())
    );

    assert_eq!(
        store
            .txn()
            .get_scoped("key1".to_owned(), Some(Path::new("/component1")))
            .await
            .unwrap(),
        Some("value2".to_owned())
    );
}

// Closing the database and opening it again should
// result in the old values being added to the database again
#[wasm_bindgen_test]
async fn persistance() {
    let test_path = PathBuf::from("/tmp/3");
    let mut store = KvStore::open(&test_path).await.unwrap();

    store.txn().set("key4".to_owned(), "value1").await.unwrap();

    drop(store);

    let mut store = KvStore::open(&test_path).await.unwrap();
    assert_eq!(
        store.txn().get("key4".to_owned()).await.unwrap(),
        Some("value1".to_owned())
    );
}

// Should get `None` when getting a non-existent key
#[wasm_bindgen_test]
async fn get_non_existent_value() {
    let test_path = PathBuf::from("/tmp/4");
    let mut store = KvStore::open(&test_path).await.unwrap();

    store.txn().set("key1".to_owned(), "value1").await.unwrap();
    assert_eq!(store.txn().get("key2".to_owned()).await.unwrap(), None);

    // Open from disk again and check persistent data
    drop(store);
    let mut store = KvStore::open(&test_path).await.unwrap();
    assert_eq!(store.txn().get("key2".to_owned()).await.unwrap(), None);
}

#[wasm_bindgen_test]
async fn remove_non_existent_key() {
    let test_path = PathBuf::from("/tmp/5");
    let mut store = KvStore::open(&test_path).await.unwrap();
    assert!(store.txn().remove("keybad".to_owned()).await.is_err());
}

#[wasm_bindgen_test]
async fn remove_key() {
    let test_path = PathBuf::from("/tmp/6");
    let mut store = KvStore::open(&test_path).await.unwrap();
    store.txn().set("key1".to_owned(), "value1").await.unwrap();
    assert_eq!(
        store.txn().get("key1".to_owned()).await.unwrap(),
        Some("value1".into())
    );
    assert!(store.txn().remove("key1".to_owned()).await.is_ok());
}

// Insert data until total size of the directory decreases.
// Test data correctness after compaction.
#[wasm_bindgen_test]
async fn compaction() {
    let test_path = PathBuf::from("/tmp/compaction");
    let mut store = KvStore::open(&test_path).await.unwrap();

    let dir_size = async || {
        let mut idb_folder = IdbFolder::open(&test_path).await.unwrap();
        let idb_file_names: Vec<String> = idb_folder
            .get_file_names()
            .await
            .expect("Could not get file names")
            .into_serde()
            .expect("File names can not be converted to vector");

        let mut sum = 0;
        for file_name in idb_file_names {
            let file = &idb_folder.open_file(Path::new(&file_name)).await.unwrap();
            sum += file.size();
        }
        sum
    };

    let mut current_size = dir_size().await;

    for iter in 0..128 {
        for key_id in 0..128 {
            let key = format!("key{}", key_id);
            let value = format!("{}", iter);
            store.txn().set(key, &value).await.unwrap();
        }
        let new_size = dir_size().await;
        if new_size > current_size {
            current_size = new_size;
            continue;
        }
        // Compaction triggered

        drop(&store);
        // reopen and check content
        let mut store = KvStore::open(&test_path).await.unwrap();
        for key_id in 0..64 {
            let key = format!("key{}", key_id);
            let res = store.txn().get(key).await.unwrap();
            assert_eq!(res, Some(format!("{}", iter)));
        }

        return ();
    }

    panic!("No compaction detected");
}
