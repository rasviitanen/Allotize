use std::collections::{BTreeMap, HashMap};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Deserializer;
use std::collections::HashSet;
use std::ops::Range;
use std::ops::{Bound, Deref, RangeBounds};

use crate::{IdbFile, IdbFolder, KvsError, Result};

const COMPACTION_THRESHOLD: u64 = 128 * 128;

pub struct KvTxn<'a> {
    /// Holds the request response for a request to `Indexeddb`
    inner: &'a mut KvStore,
    modified_stores: HashSet<Option<PathBuf>>, // TODO We can probably be smarter here
}

impl<'a> KvTxn<'a> {
    pub fn new(inner: &'a mut KvStore) -> Self {
        KvTxn {
            inner,
            modified_stores: HashSet::new(),
        }
    }

    pub async fn set<T: ?Sized + Serialize>(&mut self, key: String, value: &T) -> Result<()> {
        self.modified_stores.insert(None);
        self.inner.set(key, serde_json::to_string(value)?).await
    }

    pub async fn set_scoped(
        &mut self,
        key: String,
        value: String,
        substore: Option<&Path>,
    ) -> Result<()> {
        substore.map(|path| self.modified_stores.insert(Some(path.to_path_buf())));
        self.inner.set_scoped(key, value, substore).await
    }

    pub async fn get(&mut self, key: String) -> Result<Option<String>> {
        self.inner.get(key).await
    }

    pub async fn get_range(
        &mut self,
        start: Bound<String>,
        end: Bound<String>,
    ) -> Result<Vec<(String, String)>> {
        self.inner.get_range((start, end)).await
    }

    pub async fn get_scoped(
        &mut self,
        key: String,
        substore: Option<&Path>,
    ) -> Result<Option<String>> {
        self.inner.get_scoped(key, substore).await
    }

    pub async fn remove(&mut self, key: String) -> Result<()> {
        self.inner.remove(key).await
    }
}

impl<'a> std::ops::Drop for KvTxn<'a> {
    fn drop(&mut self) {
        for path in &self.modified_stores {
            self.inner.flush(path.as_ref());
        }
    }
}

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are persisted to `indexeddb` in log files. Log files are named after
/// monotonically increasing generation numbers with a `log` extension name.
/// A `BTreeMap` in memory stores the keys and the value locations for fast query.
pub struct KvStore {
    // sink for the data to be stored, here we use `IndexedDB`
    sink: IdbFolder,
    // directory for the log and other data
    path: PathBuf,
    // map generation number to the file reader
    readers: HashMap<PathBuf, BufReaderWithPos<IdbFile>>,
    // writer of the current log
    writers: HashMap<PathBuf, BufWriterWithPos<IdbFile>>,

    current_gen: u64,
    index: BTreeMap<String, CommandPos>,
    // the number of bytes representing "stale" commands that could be
    // deleted during a compaction
    uncompacted: u64,
}

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    ///
    /// # Errors
    ///
    /// It propagates I/O or deserialization errors during the log replay.
    pub async fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        let mut sink = IdbFolder::open(&path).await.expect("Could not create sink");

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();

        // let remote_gen_list = {
        //     let window = web_sys::window().unwrap();
        //     let mut request = web_sys::RequestInit::new();
        //     request.method("GET");
        //     let promise = window.fetch_with_str_and_init(
        //         &format!("http://127.0.0.1:8787/list_records/{}", path.display()),
        //         &request,
        //     );
        //     let response = wasm_bindgen_futures::JsFuture::from(promise)
        //         .await
        //         .expect("oops");
        //     let response = web_sys::Response::from(response);
        // };

        let gen_list = sorted_gen_list(&path).await?;
        let mut uncompacted = 0;

        for &gen in &gen_list {
            let log_path = log_path(&path, gen);
            let file = sink.open_file(&log_path).await?;
            let mut reader = BufReaderWithPos::new(file)?;
            uncompacted += load(gen, &mut reader, &mut index)?;
            readers.insert(log_path, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, &mut sink, current_gen, &mut readers).await?;
        let mut writers = HashMap::new();
        writers.insert(path.clone(), writer);

        Ok(KvStore {
            sink,
            path,
            readers,
            writers,
            current_gen,
            index,
            uncompacted,
        })
    }

    fn flush(&mut self, subpath: Option<&PathBuf>) {
        let mut path = PathBuf::new();
        path.push(&self.path);
        if let Some(p) = subpath {
            path.push(&p);
        }

        let writer = self.writers.get_mut(&path).expect("Could not get writer");
        writer.flush().expect("Could not flush");
    }

    /// Returns a txn, that when dropped will flush all the transactions to idb
    pub fn txn(&mut self) -> KvTxn {
        KvTxn::new(self)
    }

    /// Adds a file to the KV-store, this makes it easy to scope
    /// different components to a separate file
    pub async fn add_substore(&mut self, sub_path: &Path) -> Result<()> {
        let mut path = PathBuf::new();
        path.push(&self.path);
        path.push(&sub_path);

        let gen_list = sorted_gen_list(&path).await?;

        for &gen in &gen_list {
            let log_path = log_path(&path, gen);
            let mut reader = BufReaderWithPos::new(self.sink.open_file(&log_path).await?)?;
            self.uncompacted += load(gen, &mut reader, &mut self.index)?;
            self.readers.insert(log_path, reader);
        }

        // TODO this should be + 1
        // let current_gen = gen_list.last().unwrap_or(&0) + 0;
        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, &mut self.sink, current_gen, &mut self.readers).await?;
        self.writers.insert(path.clone(), writer);

        Ok(())
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Errors
    ///
    /// It propagates I/O or serialization errors during writing the log.
    async fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let mut writer = self
            .writers
            .get_mut(&self.path)
            .expect("Could not get writer");
        let pos = writer.pos;
        serde_json::to_writer(&mut writer, &cmd)?;

        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self
                .index
                .insert(key, (self.current_gen, pos..writer.pos).into())
            {
                self.uncompacted += old_cmd.len;
            }
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact().await?;
        }

        Ok(())
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Errorsset_scoped
    ///
    /// It propagates I/O or serialization errors during writing the log.
    async fn set_scoped(
        &mut self,
        key: String,
        value: String,
        substore: Option<&Path>,
    ) -> Result<()> {
        let cmd = Command::set(key, value);
        let mut path = PathBuf::new();
        path.push(&self.path);
        if let Some(p) = substore {
            path.push(&p);
        }

        let mut writer = self.writers.get_mut(&path).expect("Could not get writer");
        let pos = writer.pos;
        serde_json::to_writer(&mut writer, &cmd)?;

        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self
                .index
                .insert(key, (self.current_gen, pos..writer.pos).into())
            {
                self.uncompacted += old_cmd.len;
            }
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact().await?;
        }

        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    async fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            let log_path = log_path(&self.path, cmd_pos.gen);

            let reader = self
                .readers
                .get_mut(&log_path)
                .expect("Cannot find log reader in get");
            reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = reader.take(cmd_pos.len);

            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// Gets all values from the store
    pub async fn get_all(&mut self) -> Result<Vec<(String, String)>> {
        let mut items = Vec::new();
        for (key, cmd_pos) in &self.index {
            let log_path = log_path(&self.path, cmd_pos.gen);

            let reader = self
                .readers
                .get_mut(&log_path)
                .expect("Cannot find log reader in get");
            reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = reader.take(cmd_pos.len);

            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                items.push((key.clone(), value));
            }
        }
        Ok(items)
    }

    /// Gets all values from the store with a given prefix
    pub async fn get_range<R: RangeBounds<String>>(
        &mut self,
        range: R,
    ) -> Result<Vec<(String, String)>> {
        let path = &self.path;
        let Self { readers, index, .. } = self;
        index
            .range(range)
            .map(|(key, cmd_pos)| {
                let log_path = log_path(path, cmd_pos.gen);

                let reader = readers.get_mut(&log_path).expect("failed to find reader");
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
                let cmd_reader = reader.take(cmd_pos.len);

                if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                    Ok((key.clone(), value))
                } else {
                    Err(KvsError::UnexpectedCommandType)
                }
            })
            .collect()
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    async fn get_scoped(&mut self, key: String, substore: Option<&Path>) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            let mut path = PathBuf::new();
            path.push(&self.path);
            if let Some(p) = substore {
                path.push(&p);
            }
            let log_path = log_path(&path, cmd_pos.gen);

            let reader = self
                .readers
                .get_mut(&log_path)
                .expect("Cannot find log reader in get scoped");
            reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = reader.take(cmd_pos.len);

            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// Removes a given key.
    ///
    /// # Errors
    ///
    /// It returns `KvsError::KeyNotFound` if the given key is not found.
    ///
    /// It propagates I/O or serialization errors during writing the log.
    async fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            let mut writer = self
                .writers
                .get_mut(&self.path)
                .expect("Could not get writer");
            serde_json::to_writer(&mut writer, &cmd)?;
            writer.flush()?;
            if let Command::Remove { key } = cmd {
                let old_cmd = self.index.remove(&key).expect("key not found");
                self.uncompacted += old_cmd.len;
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }

    /// Clears stale entries in the log.
    async fn compact(&mut self) -> Result<()> {
        // increase current gen by 2. current_gen + 1 is for the compaction file
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        let writer = self.new_log_file(self.current_gen).await?;
        self.writers.insert(self.path.clone(), writer);

        let mut compaction_writer = self.new_log_file(compaction_gen).await?;

        let mut new_pos = 0; // pos in the new log file
        for cmd_pos in &mut self.index.values_mut() {
            let log_path = log_path(&self.path, cmd_pos.gen);
            let reader = self
                .readers
                .get_mut(&log_path)
                .expect("Cannot find log reader");
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut compaction_writer)?;
            *cmd_pos = (compaction_gen, new_pos..new_pos + len).into();
            new_pos += len;
        }
        compaction_writer.flush()?;

        // remove stale log files
        let stale_gens: Vec<_> = self
            .readers
            .keys()
            .filter(|path| {
                path.file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
                    < compaction_gen
            })
            .cloned()
            .collect();

        for stale_gen in stale_gens {
            self.readers.remove(&stale_gen);
            if self.sink.remove_file(&stale_gen).await.is_err() {
                info!("Could not remove stale gen {}", stale_gen.display());
            };
        }

        self.uncompacted = 0;

        Ok(())
    }

    /// Create a new log file with given generation number and add the reader to the readers map.
    ///
    /// Returns the writer to the log.
    async fn new_log_file(&mut self, gen: u64) -> Result<BufWriterWithPos<IdbFile>> {
        new_log_file(&self.path, &mut self.sink, gen, &mut self.readers).await
    }
}

/// Create a new log file with given generation number and add the reader to the readers map.
///
/// Returns the writer to the log.
async fn new_log_file(
    path: &Path,
    sink: &mut IdbFolder,
    gen: u64,
    readers: &mut HashMap<PathBuf, BufReaderWithPos<IdbFile>>,
) -> Result<BufWriterWithPos<IdbFile>> {
    let path = log_path(&path, gen);
    let writer = BufWriterWithPos::new(sink.open_file(&path).await?)?;
    readers.insert(
        path.clone(),
        BufReaderWithPos::new(sink.open_file(&path).await?)?,
    );
    Ok(writer)
}

/// Returns sorted generation numbers in the given directory
async fn sorted_gen_list(folder_path: &Path) -> Result<Vec<u64>> {
    // TODO Get all items in an object store
    let path: PathBuf = folder_path.into();

    let idb_folder = IdbFolder::open(&path).await?;
    let idb_files: Vec<String> = idb_folder
        .get_file_names()
        .await
        .expect("Could not get file names")
        .into_serde()
        .expect("File names can not be converted to vector");

    let filtered: Vec<_> = idb_files
        .iter()
        .filter(|file_name| file_name.contains(folder_path.to_str().unwrap()))
        .map(|file_name| Path::new(file_name))
        .collect();

    let mut gen_list: Vec<u64> = filtered
        .iter()
        .map(Deref::deref)
        .filter_map(Path::file_stem)
        .filter_map(|file| file.to_str())
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    gen_list.sort_unstable();
    Ok(gen_list)
}

/// Load the whole log file and store value locations in the index map.
///
/// Returns how many bytes can be saved after a compaction.
fn load(
    gen: u64,
    reader: &mut BufReaderWithPos<IdbFile>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    // To make sure we read from the beginning of the file
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0; // number of bytes that can be saved after a compaction
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        // Check if the command is successfully read
        match cmd.ok() {
            Some(Command::Set { key, .. }) => {
                if let Some(old_cmd) = index.insert(key, (gen, pos..new_pos).into()) {
                    uncompacted += old_cmd.len;
                }
            }
            Some(Command::Remove { key }) => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.len;
                }
                // the "remove" command itself can be deleted in the next compaction
                // so we add its length to `uncompacted`
                uncompacted += new_pos - pos;
            }
            _ => {
                // A false read has occured if we reach this.
                // A false read occurs  if the database is closed
                // before all bytes are flushed
                break;
            }
        }
        pos = new_pos;
    }

    Ok(uncompacted)
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{}", gen))
}

/// Struct representing a command
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

/// Represents the position and length of a json-serialized command in the log
#[derive(Debug, Clone, Copy)]
struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((gen, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            gen,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

#[derive(Debug)]
struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
