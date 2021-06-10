use std::sync::{Arc, RwLock};

use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys;

use std::cmp;
use std::collections::HashMap;

use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::pin::Pin;

#[derive(Default, Debug, Serialize, Deserialize)]
struct RawFile {
    pub inner: Vec<u8>,
}

/// Emulates a Folder that is stored in `IndexedDB`
pub struct IdbFolder {
    _name: Arc<String>,
    idb_handle: Arc<IdbHandle>,
    raw_files: HashMap<String, Arc<RwLock<RawFile>>>,
}

impl IdbFolder {
    /// Creates a file that gets stored in memory
    pub async fn open(path: &Path) -> io::Result<IdbFolder> {
        let idb_handle = IdbOpenDbRequest::new("hadal-db")
            .open_with_store("hadal-store")
            .await
            .expect("Could not open store");

        let name = path.to_str().expect("Could not transform path to str");

        Ok(IdbFolder {
            _name: Arc::new(name.into()),
            idb_handle: Arc::new(idb_handle),
            raw_files: HashMap::new(),
        })
    }

    /// Returns the names of all stored files
    pub async fn get_file_names(&self) -> Result<JsValue, JsValue> {
        self.idb_handle.get_all_keys().await
    }

    /// Removes a file for idb
    pub async fn remove_file(&self, path: &Path) -> Result<JsValue, JsValue> {
        let name = path.to_str().expect("Could not transform path to str");
        let handle = &self.idb_handle;
        handle.remove(name).await
    }

    /// Opens a new file in idb
    pub async fn open_file(&mut self, path: &Path) -> io::Result<IdbFile> {
        let name = path.to_str().expect("Could not transform path to str");
        let file_as_jsv: Result<JsValue, JsValue> = self.idb_handle.get(name).await;

        let raw_file: RawFile = match file_as_jsv {
            Ok(file) => {
                if file.is_undefined() {
                    RawFile::default()
                } else {
                    file.into_serde().expect("Could not serialize file")
                }
            }
            Err(_err) => RawFile::default(),
        };

        let pos = raw_file.inner.len() as u64;

        let file = match self.raw_files.get(name) {
            Some(file) => Arc::clone(&file),
            _ => {
                let file = Arc::new(RwLock::new(raw_file));
                self.raw_files.insert(name.into(), Arc::clone(&file));
                file
            }
        };

        Ok(IdbFile {
            pos,
            name: name.into(),
            idb_handle: Arc::clone(&self.idb_handle),
            inner: file,
        })
    }
}

/// Emulates a File that is stored in `IndexedDB`
pub struct IdbFile {
    pos: u64,
    name: String,
    idb_handle: Arc<IdbHandle>,
    inner: Arc<RwLock<RawFile>>,
}

impl IdbFile {
    /// Creates a file that gets stored in memory
    // pub async fn open(path: &Path) -> io::Result<IdbFile> {
    //     let idb_handle = IdbOpenDbRequest::new("hadal-db")
    //         .open_with_store("hadal-store")
    //         .await
    //         .expect("Could not open store");

    //     let name = path.to_str().expect("Could not transform path to str");

    //     let file_as_jsv: Result<JsValue, JsValue> = idb_handle.get(name).await;

    //     let raw_file: RawFile = match file_as_jsv {
    //         Ok(file) => {
    //             if file.is_undefined() {
    //                 RawFile::default()
    //             } else {
    //                 file.into_serde().expect("Could not serialize file")
    //             }
    //         }
    //         Err(_err) => RawFile::default(),
    //     };

    //     let pos = raw_file.inner.len() as u64;

    //     let mut files = GLOBAL_FILE_STORAGE.lock().expect("Could not lock files");
    //     let file = match files.get(name.into()) {
    //         Some(file) => Arc::clone(&file),
    //         _ => {
    //             let file = Arc::new(RwLock::new(raw_file));
    //             files.insert(name.into(), Arc::clone(&file));
    //             file
    //         }
    //     };

    //     Ok(IdbFile {
    //         pos,
    //         name: Arc::new(name.into()),
    //         idb_handle: Arc::new(idb_handle),
    //         inner: file,
    //     })
    // }

    /// Returns the size of a file in bytes
    pub fn size(&self) -> usize {
        self.inner.read().unwrap().inner.len()
    }

    /// Saves a file to idb
    pub async fn save(&mut self) {
        let raw_file: &RawFile = &*(self.inner.read().unwrap());
        self.idb_handle.put(&self.name, raw_file);
    }
}

impl Read for IdbFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(self.pos, self.inner.read().unwrap().inner.len() as u64);
        let mut fill_buff = &self.inner.read().unwrap().inner[(amt as usize)..];
        let n = Read::read(&mut fill_buff, buf)?;
        self.pos += n as u64;

        Ok(n)
    }
}

impl Write for IdbFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let pos: usize = self.pos as usize;
        let len: usize = self
            .inner
            .read()
            .expect("Could not get read lock on raw file")
            .inner
            .len();
        // Make sure the internal buffer is as least as big as where we
        // currently are
        if len < pos {
            // use `resize` so that the zero filling is as efficient as possible
            self.inner
                .write()
                .expect("Could not get write lock on raw file")
                .inner
                .resize(pos, 0);
        }

        // Figure out what bytes will be used to overwrite what's currently
        // there (left), and what will be appended on the end (right)
        {
            let space = self
                .inner
                .read()
                .expect("Could not get read lock on raw file")
                .inner
                .len()
                - pos;
            let (left, right) = buf.split_at(cmp::min(space, buf.len()));
            self.inner
                .write()
                .expect("Could not get write lock on raw file")
                .inner[pos..pos + left.len()]
                .copy_from_slice(left);
            self.inner
                .write()
                .expect("Could not get write lock on raw file")
                .inner
                .extend_from_slice(right);
        }

        // Bump us forward
        self.pos = (pos + buf.len()) as u64;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Save to indexed db
        let file = Arc::clone(&self.inner);
        let idb = Arc::clone(&self.idb_handle);
        let name = self.name.clone();

        let future = async move {
            let raw_file: &RawFile = &*(file.read().expect("Could not get read lock on raw file"));
            idb.put(&name, raw_file);
        };

        wasm_bindgen_futures::spawn_local(future);

        Ok(())
    }
}

impl Seek for IdbFile {
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.read().unwrap().inner.len() as u64, n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as u64)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as u64)
        };
        match new_pos {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "Uh oh")),
        }
    }
}

pub struct IdbTxn {
    /// Holds the request response for a request to `Indexeddb`
    pub inner: Arc<web_sys::IdbRequest>,
    callback: Option<Closure<dyn FnMut()>>,
}

impl IdbTxn {
    pub fn new(request: web_sys::IdbRequest) -> IdbTxn {
        IdbTxn {
            inner: Arc::new(request),
            callback: None,
        }
    }
}

impl std::future::Future for IdbTxn {
    type Output = Result<JsValue, JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        use web_sys::IdbRequestReadyState as ReadyState;
        match self.inner.ready_state() {
            ReadyState::Pending => {
                let task = cx.waker().clone();
                self.callback = Some(Closure::once(move || task.wake()));

                self.inner
                    .set_onsuccess(self.callback.as_ref().map(|c| c.as_ref().unchecked_ref()));
                self.inner
                    .set_onerror(self.callback.as_ref().map(|c| c.as_ref().unchecked_ref()));

                Poll::Pending
            }
            ReadyState::Done => match self.inner.result() {
                Ok(val) => Poll::Ready(Ok(val)),
                Err(e) => Poll::Ready(Err(e)),
            },
            _ => {
                info!("Unexpected Ready state", "When polling IdbTxn");
                Poll::Ready(Err(JsValue::from_str("Error in transaction")))
            }
        }
    }
}

/// A handle to acces an `Indexeddb`
#[derive(Debug)]
pub struct IdbHandle {
    pub inner: web_sys::IdbDatabase,
    active_store: String,
}

impl IdbHandle {
    fn new(inner: web_sys::IdbDatabase, active_store: String) -> Self {
        Self {
            inner,
            active_store,
        }
    }

    /// Sets the keys of an object store.
    pub fn get_all_keys(&self) -> IdbTxn {
        IdbTxn::new(
            self.inner
                .transaction_with_str_and_mode(
                    &self.active_store,
                    web_sys::IdbTransactionMode::Readwrite,
                )
                .unwrap()
                .object_store(&self.active_store)
                .unwrap()
                .get_all_keys()
                .unwrap(),
        )
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Errors
    ///
    /// It propagates I/O or serialization errors during writing the log.
    pub fn put<T>(&self, key: &str, value: &T) -> IdbTxn
    where
        T: serde::ser::Serialize + ?Sized,
    {
        let key_as_jsv =
            JsValue::from_serde(&key.to_string()).expect("Unable to serialize to JsValue");
        let value_as_jsv = JsValue::from_serde(&value).expect("Unable to serialize to JsValue");

        IdbTxn::new(
            self.inner
                .transaction_with_str_and_mode(
                    &self.active_store,
                    web_sys::IdbTransactionMode::Readwrite,
                )
                .unwrap()
                .object_store(&self.active_store)
                .unwrap()
                .put_with_key(&value_as_jsv, &key_as_jsv)
                .unwrap(),
        )
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&self, key: &str) -> IdbTxn {
        let key_as_jsv =
            JsValue::from_serde(&key.to_string()).expect("Unable to serialize to JsValue");

        let store = self
            .inner
            .transaction_with_str_and_mode(
                &self.active_store,
                web_sys::IdbTransactionMode::Readonly,
            )
            .expect("Could not create transaction with str_and_mode")
            .object_store(&self.active_store)
            .expect("Could not create object store");

        IdbTxn::new(store.get(&key_as_jsv).expect("No such key"))
    }

    /// Removes a given key.
    ///
    /// # Errors
    ///
    /// It returns `KvsError::KeyNotFound` if the given key is not found.
    ///
    /// It propagates I/O or serialization errors during writing the log.
    pub fn remove(&self, key: &str) -> IdbTxn {
        let key_as_jsv =
            JsValue::from_serde(&key.to_string()).expect("Unable to serialize to JsValue");
        IdbTxn::new(
            self.inner
                .transaction_with_str_and_mode(
                    &self.active_store,
                    web_sys::IdbTransactionMode::Readwrite,
                )
                .expect("Could not create transaction")
                .object_store(&self.active_store)
                .expect("Could not get hold of object store")
                .delete(&key_as_jsv)
                .expect("Could not remove the given key"),
        )
    }
}

/// Holds an indexed database
pub struct IdbOpenDbRequest {
    inner: Arc<web_sys::IdbOpenDbRequest>,
    callback: Option<Closure<dyn FnMut()>>,
}

impl IdbOpenDbRequest {
    /// Creates a new request to open a database
    pub fn new(name: &str) -> IdbOpenDbRequest {
        let window = web_sys::window().unwrap();
        let idb_factory = window.indexed_db().unwrap().expect("Idb not supported");
        let open_request = idb_factory
            .open(name)
            .expect("TypeError is not possible with Rust");

        IdbOpenDbRequest {
            inner: Arc::new(open_request),
            callback: None,
        }
    }

    /// Opens a new database
    pub fn open_with_store(self, name: &str) -> IdbOpenDbRequest {
        let _db_request_aclone = Arc::clone(&self.inner);

        let name_clone: String = name.into();
        let upgradeneeded_cb = Closure::once(move |e: web_sys::Event| {
            // Get the database
            // We will only use one version, so versioning is not required
            let target = e.target().expect("event should have a target");
            let req = target
                .dyn_ref::<web_sys::IdbRequest>()
                .expect("target should be IdbRequest");
            let result = req.result().expect("IdbRequest should have a result");
            let db: &web_sys::IdbDatabase = result.unchecked_ref();

            let object_store = db
                .create_object_store(&name_clone)
                .expect("Could not create object store");
            object_store
                .create_index_with_str("log_files", "log_file")
                .expect("Could not create index in object store");
        });

        // set message event handler on the database
        // and forget the callback to keep it alive
        self.inner
            .set_onupgradeneeded(Some(upgradeneeded_cb.as_ref().unchecked_ref()));
        upgradeneeded_cb.forget();

        self
    }
}

impl std::future::Future for IdbOpenDbRequest {
    type Output = Result<IdbHandle, JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        use web_sys::IdbRequestReadyState as ReadyState;
        match self.inner.ready_state() {
            ReadyState::Pending => {
                let task = cx.waker().clone();
                self.callback = Some(Closure::once(move || task.wake()));

                self.inner
                    .set_onsuccess(self.callback.as_ref().map(|c| c.as_ref().unchecked_ref()));
                self.inner
                    .set_onerror(self.callback.as_ref().map(|c| c.as_ref().unchecked_ref()));

                Poll::Pending
            }
            ReadyState::Done => match self.inner.result() {
                Ok(val) => Poll::Ready(Ok(IdbHandle::new(
                    val.unchecked_into(),
                    "hadal-store".into(),
                ))),
                Err(e) => Poll::Ready(Err(e)),
            },
            _ => Poll::Ready(Err(JsValue::from_str("Could not get hold of store"))),
        }
    }
}
