use crate::{net_traits::AppMetadata, Identity, RtcMessage, RtcPool, RtcTxn};
use allotize_db::KvStore;
use futures::lock::Mutex;
use std::ops::Bound;
use std::path::PathBuf;
use std::sync::Arc;

use js_sys::Object;
use js_sys::Proxy;
use wasm_bindgen::prelude::*;
use web_sys::{CustomEvent, EventTarget, MessageEvent};

use crate::net_traits::{JSVal, VersionedComponent};
use crdts::CvRDT;

#[wasm_bindgen]
pub struct Tx {
    _scope: Option<String>,
    event_target: Arc<EventTarget>,
    pool: Arc<Mutex<RtcPool>>,
    store: Arc<Mutex<KvStore>>,
    identity: Identity,
}

fn notify_js_about_local_change(event_target: &EventTarget, key: &str, component: &VersionedComponent) {
    let key = format!("{}@local", key);
    let notify_event = CustomEvent::new(&key).unwrap();
    notify_event.init_custom_event_with_can_bubble_and_cancelable_and_detail(
        &key,
        true,
        true,
        &JsValue::from_serde(component).unwrap()
    );
    event_target
        .dispatch_event(&notify_event)
        .expect("Could not dispatch event");
}

#[wasm_bindgen]
impl Tx {
    /// Shares a key/value pair with connected users
    pub fn share(&self, key: String, value: JsValue) -> js_sys::Promise {
        let pool = Arc::clone(&self.pool);
        let future = async move {
            let message = RtcMessage {
                command: "share".into(),
                key: key.clone(),
                value: value.into_serde().ok(),
            };

            pool.lock().await.txn().broadcast(&message).await;
            Ok(JsValue::from_str("OK"))
        };
        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Puts a key/volue pair in the store and notifies connected
    /// peers about the change.
    ///
    /// If no peer is connected, it postpones the message until
    /// atleast one peer is listening.
    pub fn put(&self, key: String, value: JsValue) -> js_sys::Promise {
        let store = Arc::clone(&self.store);
        let pool = Arc::clone(&self.pool);
        let event_target = Arc::clone(&self.event_target);
        let future = async move {
            store
                .lock()
                .await
                .txn()
                .set(key.clone(), &JSVal { v: value.clone() })
                .await
                .unwrap();

            let component = value.into_serde().ok();

            if let Some(component) = &component {
                notify_js_about_local_change(&event_target, &key, &component);
            }

            let message = RtcMessage {
                command: "put".into(),
                key,
                value: component,
            };

            pool.lock().await.require_channels(1).await.unwrap();
            pool.lock().await.txn().broadcast(&message).await;
            Ok(JsValue::from_str("OK"))
        };
        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Puts a key/value pair in the store using CRDT. After
    /// applying the changes, the peers are notified.
    ///
    /// If no peer is connected, it postpones the message until
    /// atleast one peer is listening.
    #[wasm_bindgen(js_name = crdtPut)]
    pub fn crdt_put(&self, key: String, value: String) -> js_sys::Promise {
        info!(
            "crdt_put123",
            "->",
            format!("key: {}, value: {:?}", key, value)
        );
        let store = Arc::clone(&self.store);
        let pool = Arc::clone(&self.pool);
        let event_target = Arc::clone(&self.event_target);
        let identity = self.identity.clone();
        let future = async move {
            // Get the old component from the store
            let mut component: VersionedComponent = store
                .lock()
                .await
                .txn()
                .get(key.clone())
                .await
                .ok()
                .flatten()
                .map(|ss| serde_json::from_str(&ss).ok())
                .flatten()
                .unwrap_or_default();

            component.apply(identity.username);
            component.data = Some(value.clone());

            store
                .lock()
                .await
                .txn()
                .set(key.clone(), &component)
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            notify_js_about_local_change(&event_target, &key, &component);

            // Notify peers about the change
            let message = RtcMessage {
                command: "crdt_put".into(),
                key,
                value: Some(component),
            };

            pool.lock().await.txn().broadcast(&message).await;
            Ok(JsValue::from_str("OK"))
        };
        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Gets a key/value pair from the store.
    ///
    /// If no value corresponds to the given key, an `JsValue`
    /// is returned containing "Not found"
    #[wasm_bindgen(js_name = crdtGet)]
    pub fn crdt_get(&self, key: String) -> js_sys::Promise {
        let store = Arc::clone(&self.store);

        let future = async move {
            let local_component: VersionedComponent = store
                .lock()
                .await
                .txn()
                .get(key.clone())
                .await
                .ok()
                .flatten()
                .map(|ss| {
                    console_log!("[request] got: {:?}", &ss);
                    serde_json::from_str(&ss).ok()
                })
                .flatten()
                .unwrap_or_default();
            Ok(local_component.data.into())
        };

        wasm_bindgen_futures::future_to_promise(future)
    }

    #[wasm_bindgen(js_name = syncWithPeers)]
    pub fn sync_with_peers(&self, key: String) -> js_sys::Promise {
        let store = Arc::clone(&self.store);
        let pool = Arc::clone(&self.pool);

        let future = async move {
            let local_component: VersionedComponent = store
                .lock()
                .await
                .txn()
                .get(key.clone())
                .await
                .ok()
                .flatten()
                .map(|ss| serde_json::from_str(&ss).ok())
                .flatten()
                .unwrap_or_default();

            // Send the current version of the component to our peers,
            // If there is a more recent one, ours will be updated,
            // otherwise theirs will
            let message = RtcMessage {
                command: "crdt_put".into(),
                key,
                value: Some(local_component.clone()),
            };

            // Notify peers about our version,
            // If we are behind, our version will be updated,
            // otherwise, theirs will
            pool.lock().await.require_channels(1).await.unwrap();
            pool.lock().await.txn().broadcast(&message).await;
            Ok(local_component.data.into())
        };

        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Gets a key/value pair from the store.
    ///
    /// If no value corresponds to the given key, an `JsValue`
    /// is returned containing "Not found"
    pub fn get(&self, key: String) -> js_sys::Promise {
        let store = Arc::clone(&self.store);
        let future = async move {
            store
                .lock()
                .await
                .txn()
                .get(key.to_owned())
                .await
                .ok()
                .flatten()
                .map(|val| JsValue::from_str(&val))
                .ok_or_else(|| JsValue::from_str("Not found"))
        };
        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Gets key/value pairs from the store for a given `Range`.
    #[wasm_bindgen(js_name = getRange)]
    pub fn get_range(&self, start: String, end: Option<String>) -> js_sys::Promise {
        let store = Arc::clone(&self.store);
        let future = async move {
            store
                .lock()
                .await
                .txn()
                .get_range(
                    Bound::Included(start),
                    end.map(|e| Bound::Excluded(e)).unwrap_or(Bound::Unbounded),
                )
                .await
                .map(|it| JsValue::from_serde(&it))
                .expect("Could not run `get_range`")
                .map_err(|_| JsValue::from_str("Not found"))
        };
        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Gets key/value pairs from the store for a given `Range`.
    #[wasm_bindgen(js_name = beginsWith)]
    pub fn begins_with(&self, mut prefix: String) -> js_sys::Promise {
        let store = Arc::clone(&self.store);
        let future = async move {
            store
                .lock()
                .await
                .txn()
                .get_range(Bound::Included(prefix.clone()), {
                    prefix.push(char::MAX);
                    Bound::Included(prefix)
                })
                .await
                .map(|it| JsValue::from_serde(&it))
                .expect("Could not run `get_range`")
                .map_err(|_| JsValue::from_str("Not found"))
        };
        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Gets a key/value pair from the store.
    ///
    /// If no value corresponds to the given key, an `JsValue`
    /// is returned containing "Not found"
    pub fn remove(&self, key: String) -> js_sys::Promise {
        let store = Arc::clone(&self.store);
        let pool = Arc::clone(&self.pool);
        let future = async move {
            let message = RtcMessage {
                command: "remove".into(),
                key: key.clone(),
                value: None,
            };

            let res = store
                .lock()
                .await
                .txn()
                .remove(key.to_owned())
                .await
                .map(|_| JsValue::from_bool(true))
                .map_err(|_| JsValue::from_bool(false));

            if res.is_ok() {
                pool.lock().await.require_channels(1).await.unwrap();
                pool.lock().await.txn().broadcast(&message).await;
            }

            res
        };

        wasm_bindgen_futures::future_to_promise(future)
    }
}

/// The `App` consists of a pool and a store.
/// All client communication uses the `App`,
/// to store data and send messages between peers.
/// Data is persisted in browser using `IndexedDB`,
/// and communication is over a `datachannel` with `WebRTC`.
///
/// All communication with the App should be done in transactions.
/// these can be created from the App's `txn()` method.
#[wasm_bindgen]
pub struct App {
    identity: Identity,
    pool: Arc<Mutex<RtcPool>>,
    store: Arc<Mutex<KvStore>>,
    event_target: Arc<EventTarget>,
}

/// Facade for `JavaScript`, all implementations here are
/// exposed to `JavaScript` through `#[wasm_bindgen]`
#[wasm_bindgen]
impl App {
    pub fn tx(&self, scope: Option<String>) -> Tx {
        Tx {
            _scope: scope,
            event_target: Arc::clone(&self.event_target),
            pool: Arc::clone(&self.pool),
            store: Arc::clone(&self.store),
            identity: self.identity.clone(),
        }
    }

    pub fn metadata(&self) -> js_sys::Promise {
        let pool = Arc::clone(&self.pool);
        let future = async move {
            let metadata = AppMetadata {
                pool: pool.lock().await.metadata(),
            };

            JsValue::from_serde(&metadata).map_err(|_| JsValue::from_str("Failed serialization"))
        };
        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Gets all items from the store
    pub fn get_all(&self) -> js_sys::Promise {
        let store = Arc::clone(&self.store);
        let future = async move {
            store
                .lock()
                .await
                .get_all()
                .await
                .map(|it| JsValue::from_serde(&it))
                .expect("Could not run `get_all`")
                .map_err(|_| JsValue::from_str("Not found"))
        };
        wasm_bindgen_futures::future_to_promise(future)
    }

    pub fn unsubscribe(&self, key: &str, callback: &js_sys::Function) {
        self.event_target
            .remove_event_listener_with_callback(&format!("{}@local", key), callback)
            .expect("Could not add event listener with callback");

        self.event_target
            .remove_event_listener_with_callback(&format!("{}@remote", key), callback)
            .expect("Could not add event listener with callback");
    }

    pub fn subscribe(&self, key: &str, callback: &js_sys::Function) {
        self.event_target
            .add_event_listener_with_callback(&format!("{}@local", key), callback)
            .expect("Could not add event listener with callback");

        self.event_target
            .add_event_listener_with_callback(&format!("{}@remote", key), callback)
            .expect("Could not add event listener with callback");
    }

    pub fn connect(
        &self,
        path: &str,
        target: &JsValue,
        handler: &Object,
        callback: &js_sys::Function,
    ) -> Proxy {
        self.event_target
            .add_event_listener_with_callback(&format!("{}@remote", path), callback)
            .expect("Could not add event listener with callback");
        self.event_target
            .add_event_listener_with_callback(&format!("{}@local", path), callback)
            .expect("Could not add event listener with callback");
        Proxy::new(target, handler)
    }

    /// Creates a new `App` context, with an associated `RtcPool` and `storage`.
    /// The `RtcPool` and `storage` will both be available when the `App` is.
    ///
    /// A message listener is attached to the `App` that applies remote
    /// changes to the local database, and fixes eventual merge conflicts.
    #[wasm_bindgen(constructor)]
    pub async fn new(username: String, send_offer: bool) -> App {
        let identity = Identity::new(&username);
        let test_path = PathBuf::from("/tempstore");

        let store = Arc::new(Mutex::new(
            KvStore::open(&test_path)
                .await
                .expect("Could not connect to store"),
        ));

        let pool = Arc::new(Mutex::new(RtcPool::new("test-room", identity.clone())));

        let event_target = Arc::new(EventTarget::new().expect("Could not create message channel"));

        let cloned_pool = Arc::clone(&pool);
        let cloned_store = Arc::clone(&store);
        let cloned_event_target = Arc::clone(&event_target);
        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            info!("Got message", "deserializing", format!("{:?}", e.data()));
            let message_string: String = e
                .data()
                .into_serde()
                .map_err(|e| {
                    info!(
                        "failed to deserialize123",
                        "String (RtcMessage)",
                        format!("{:?}", e)
                    )
                })
                .expect("Could not deserialize message string");

            info!("Got message", "app message handler", message_string.clone());

            let rtc_message: RtcMessage = serde_json::from_str(&message_string)
                .map_err(|e| info!("failed to deserialize123", "RtcMessage", format!("{:?}", e)))
                .expect("Could not serailize message");

            info!(
                "Got message",
                "app message handler",
                format!("{:?}", &rtc_message)
            );

            // Notify
            let notify = |target: &EventTarget, key: &str, component: &VersionedComponent| {
                let key = format!("{}@remote", key);
                let notify_event = CustomEvent::new(&key).unwrap();
                notify_event.init_custom_event_with_can_bubble_and_cancelable_and_detail(
                    &key,
                    true,
                    true,
                    &JsValue::from_serde(component).unwrap());

                target
                    .dispatch_event(&notify_event)
                    .expect("Could not dispatch event");
            };

            if rtc_message.command == "share" {
                notify(&cloned_event_target, &rtc_message.key, rtc_message.value.as_ref().unwrap());
            } else if rtc_message.command == "put" {
                notify(&cloned_event_target, &rtc_message.key, rtc_message.value.as_ref().unwrap());
                let cloned_store2 = Arc::clone(&cloned_store);
                // Update the value
                wasm_bindgen_futures::spawn_local(async move {
                    cloned_store2
                        .lock()
                        .await
                        .txn()
                        .set(rtc_message.key, &rtc_message.value.unwrap())
                        .await
                        .unwrap();
                });
            } else if rtc_message.command == "remove" {
                notify(&cloned_event_target, &rtc_message.key, rtc_message.value.as_ref().unwrap());

                let cloned_store2 = Arc::clone(&cloned_store);
                wasm_bindgen_futures::spawn_local(async move {
                    cloned_store2
                        .lock()
                        .await
                        .txn()
                        .remove(rtc_message.key)
                        .await
                        .unwrap();
                })
            } else if rtc_message.command == "crdt_put" {
                let cloned_pool2 = Arc::clone(&cloned_pool);
                let cloned_store2 = Arc::clone(&cloned_store);
                let cloned_event_target2 = Arc::clone(&cloned_event_target);
                wasm_bindgen_futures::spawn_local(async move {
                    let mut local_component: VersionedComponent = cloned_store2
                        .lock()
                        .await
                        .txn()
                        .get(rtc_message.key.clone())
                        .await
                        .ok()
                        .flatten()
                        .map(|ss| serde_json::from_str(&ss).ok())
                        .flatten()
                        .unwrap_or_default();

                    let remote_component = rtc_message.value.as_ref().expect("missing remote component");

                    match local_component.clock.partial_cmp(&remote_component.clock) {
                        Some(std::cmp::Ordering::Equal) => {
                            info!(
                                "CRDT",
                                "UP TO DATE",
                                "All changes seen, accept final merge...",
                                format!(" Local: {:?}", local_component),
                                format!(" Remote: {:?}", remote_component)
                            );

                            cloned_store2
                                .lock()
                                .await
                                .txn()
                                .set(rtc_message.key.clone(), &remote_component)
                                .await
                                .unwrap()
                        }
                        Some(std::cmp::Ordering::Less) => {
                            notify(&cloned_event_target2, &rtc_message.key, rtc_message.value.as_ref().unwrap());
                            // Remote is ahead, so we trash our version
                            // and use theirs instead
                            info!(
                                "CRDT",
                                "REMOTE IS AHEAD",
                                "Updating local component...",
                                rtc_message.key.clone(),
                                ""
                            );


                            cloned_store2
                                .lock()
                                .await
                                .txn()
                                .set(rtc_message.key.clone(), &remote_component)
                                .await
                                .unwrap();
                        }
                        Some(std::cmp::Ordering::Greater) => {
                            info!(
                                "CRDT",
                                "LOCAL IS AHEAD", "Sending local component to remote..."
                            );

                            // Local is ahead
                            // so we notify our peer about this,
                            // so that they can update their data
                            let message = RtcMessage {
                                command: "crdt_put".into(),
                                key: rtc_message.key,
                                value: Some(local_component),
                            };
                            cloned_pool2.lock().await.txn().broadcast(&message).await;
                            return;
                        }
                        None => {
                            info!(
                                "CRDT",
                                "MERGE CONFLICT",
                                "Handeling merge with appropriate strategy...",
                                format!(" Local: {:?}", local_component),
                                format!(" Remote: {:?}", remote_component)
                            );

                            // Clocks are not synchronized, which means we
                            // have a merge conflict

                            // E.g. Local: `<BOB:2>`, Remote: `<ALICE:1, BOB:1>`
                            // Here we use a manual strategy to handle the
                            // merge conflict
                            // FIXME:(rasviitanen)
                            // We should use a better strategy, such
                            // as timestamping. Best of all is to allow
                            // users to specify their own strategy
                            if local_component.data < remote_component.data {
                                // make users pick the same item
                                return;
                            }

                            local_component.clock.merge(remote_component.clock.clone());
                            local_component.data = remote_component.clone().data;

                            notify(&cloned_event_target2, &rtc_message.key, rtc_message.value.as_ref().unwrap());

                            cloned_store2
                                .lock()
                                .await
                                .txn()
                                .set(rtc_message.key.clone(), &local_component)
                                .await
                                .unwrap();

                            // Notify peers about the merge change
                            let message = RtcMessage {
                                command: "crdt_put".into(),
                                key: rtc_message.key,
                                value: Some(local_component),
                            };
                            cloned_pool2.lock().await.txn().broadcast(&message).await;

                            return;
                        }
                    }
                });
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        pool.lock().await.setup(send_offer);
        pool.lock().await.set_onmessage(Some(onmessage));

        App {
            identity,
            pool,
            store,
            event_target,
        }
    }
}

impl App {
    /// Puts a KV-pair into the database, and shares the edit with connected peers
    /// so that they can update their values as well.
    pub async fn put_shared(
        &mut self,
        key: String,
        value: VersionedComponent,
    ) -> Result<JsValue, JsValue> {
        self.store
            .lock()
            .await
            .txn()
            .set(key.clone(), &value)
            .await
            .unwrap();

        // Notify peers about the change
        let message = RtcMessage {
            command: "put".into(),
            key,
            value: Some(value),
        };
        self.pool.lock().await.txn().broadcast(&message).await;

        Ok(JsValue::NULL)
    }

    /// Given a key, this function returns a `Promise`, that when resolved
    /// will return the corresponing value.
    pub async fn get_shared(&mut self, key: &str) -> Option<VersionedComponent> {
        self.store
            .lock()
            .await
            .txn()
            .get(key.to_owned())
            .await
            .ok()
            .flatten()
            .map(|value| serde_json::from_str(&value).ok())
            .flatten()
    }

    /// Puts a KV-pair into the database, and shares the edit with connected peers
    /// so that they can update their values as well.
    pub async fn crdt_put(
        &mut self,
        actor: String,
        key: String,
        value: VersionedComponent,
    ) -> Result<JsValue, JsValue> {
        let new_component = if let Some(mut component) = self.crdt_get(&key).await {
            component.apply(actor);
            component.data = value.data;
            component
        } else {
            value
        };

        self.store
            .lock()
            .await
            .txn()
            .set(key.clone(), &new_component)
            .await
            .unwrap();

        // Notify peers about the change
        let message = RtcMessage {
            command: "crdt_put".into(),
            key,
            value: Some(new_component),
        };

        self.pool.lock().await.txn().broadcast(&message).await;
        Ok(JsValue::NULL)
    }

    /// Gets a KV-pair from the database
    pub async fn crdt_get(&mut self, key: &str) -> Option<VersionedComponent> {
        let old = self
            .store
            .lock()
            .await
            .txn()
            .get(key.to_owned())
            .await
            .ok()
            .flatten();

        old.map(|ss| serde_json::from_str(&ss).ok()).flatten()
    }

    /// Enables easier syntax for waiting for the pool to be ready.
    pub async fn require_channels(&mut self, open_channels: u64) {
        self.pool
            .lock()
            .await
            .require_channels(open_channels)
            .await
            .unwrap();
    }

    pub fn identity(&self) -> Identity {
        self.identity.clone()
    }

    /// Creates a new transaction associated with the current `App`
    /// This is the main point of communication for clients,
    /// as all interaction with the app should be done using a transaction.
    pub async fn txn(&self) -> RtcTxn {
        self.pool.lock().await.txn()
    }
}
