//! Shared OPFS primitive helpers for wasm32.
//!
//! Extracted from `series/store_wasm.rs` so both `DiskStore<T>` and the
//! wasm `StorageManager` can reuse the same low-level OPFS operations.
//!
//! All functions here are `pub(crate)` — they are internal implementation
//! details, not part of the public API.

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemCreateWritableOptions, FileSystemDirectoryHandle, FileSystemFileHandle,
    FileSystemGetDirectoryOptions, FileSystemGetFileOptions, FileSystemWritableFileStream,
    WriteCommandType, WriteParams,
};

// ── re-export OpfsError so callers use one canonical type ─────────────────────

/// Errors from OPFS operations.
#[derive(Debug)]
pub enum OpfsError {
    /// `window()` returned `None` — called outside browser context.
    NoWindow,
    /// An OPFS operation returned a JS exception (`JsValue`).
    Js(wasm_bindgen::JsValue),
    /// File not found in OPFS (not a hard error; read returns empty Vec).
    FileNotFound,
    /// Day rotation failed during flush.
    RotationFailed(Box<OpfsError>),
}

impl std::fmt::Display for OpfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpfsError::NoWindow => write!(f, "OPFS: no browser window"),
            OpfsError::Js(v) => write!(f, "OPFS JS error: {:?}", v),
            OpfsError::FileNotFound => write!(f, "OPFS: file not found"),
            OpfsError::RotationFailed(inner) => {
                write!(f, "OPFS: day rotation failed: {inner}")
            }
        }
    }
}

impl std::error::Error for OpfsError {}

impl From<wasm_bindgen::JsValue> for OpfsError {
    fn from(v: wasm_bindgen::JsValue) -> Self {
        OpfsError::Js(v)
    }
}

/// Convert `OpfsError` into `std::io::Error` so wasm StorageManager can
/// satisfy the same `std::io::Result` surface used by cure and replay.
impl From<OpfsError> for std::io::Error {
    fn from(e: OpfsError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}

// ── OPFS helpers ──────────────────────────────────────────────────────────────

/// Walk (or create) a four-segment directory under `root` keyed by
/// `(stream_kind, exchange, account, symbol)`.
///
/// Path: `<root>/<stream_kind>/<exchange>/<account>/<symbol-lowercase>/`
pub(crate) async fn opfs_walk_or_create_stream(
    root: &FileSystemDirectoryHandle,
    stream_kind: &str,
    exchange: &str,
    account: &str,
    symbol: &str,
) -> Result<FileSystemDirectoryHandle, OpfsError> {
    let opts = FileSystemGetDirectoryOptions::new();
    opts.set_create(true);

    let kind_dir: FileSystemDirectoryHandle =
        JsFuture::from(root.get_directory_handle_with_options(stream_kind, &opts))
            .await?
            .dyn_into()?;

    let exch_dir: FileSystemDirectoryHandle =
        JsFuture::from(kind_dir.get_directory_handle_with_options(exchange, &opts))
            .await?
            .dyn_into()?;

    let acct_dir: FileSystemDirectoryHandle =
        JsFuture::from(exch_dir.get_directory_handle_with_options(account, &opts))
            .await?
            .dyn_into()?;

    let sym_dir: FileSystemDirectoryHandle = JsFuture::from(
        acct_dir.get_directory_handle_with_options(&symbol.to_lowercase(), &opts),
    )
    .await?
    .dyn_into()?;

    Ok(sym_dir)
}

/// Append `data` to `name` inside `dir` via `createWritable(keepExistingData:
/// true)` + seek-to-end + write + close.
pub(crate) async fn opfs_append(
    dir: &FileSystemDirectoryHandle,
    name: &str,
    data: &[u8],
) -> Result<(), OpfsError> {
    let fopts = FileSystemGetFileOptions::new();
    fopts.set_create(true);

    let fh: FileSystemFileHandle =
        JsFuture::from(dir.get_file_handle_with_options(name, &fopts))
            .await?
            .dyn_into()?;

    let file_obj: web_sys::File = JsFuture::from(fh.get_file()).await?.dyn_into()?;
    let existing_size = file_obj.size() as u64;

    let write_opts = FileSystemCreateWritableOptions::new();
    write_opts.set_keep_existing_data(true);

    let writable: FileSystemWritableFileStream =
        JsFuture::from(fh.create_writable_with_options(&write_opts))
            .await?
            .dyn_into()?;

    let seek_params = WriteParams::new(WriteCommandType::Seek);
    seek_params.set_position(Some(existing_size as f64));
    JsFuture::from(
        writable
            .write_with_write_params(&seek_params)
            .map_err(OpfsError::Js)?,
    )
    .await?;

    let arr = js_sys::Uint8Array::from(data);
    JsFuture::from(
        writable
            .write_with_buffer_source(&arr)
            .map_err(OpfsError::Js)?,
    )
    .await?;

    JsFuture::from(writable.close()).await?;
    Ok(())
}

/// Wrapper around `opfs_append` that catches `QuotaExceededError` and logs a
/// warning instead of propagating it. Any other error is returned.
pub(crate) async fn opfs_append_quota_guard(
    dir: &FileSystemDirectoryHandle,
    name: &str,
    data: &[u8],
) -> Result<(), OpfsError> {
    match opfs_append(dir, name, data).await {
        Ok(()) => Ok(()),
        Err(OpfsError::Js(ref jsval)) => {
            let is_quota = js_sys::Reflect::get(jsval, &"name".into())
                .ok()
                .and_then(|v| v.as_string())
                .map(|n| n == "QuotaExceededError")
                .unwrap_or(false);
            if is_quota {
                tracing::warn!(
                    target: "dig3_station::opfs",
                    file = name,
                    bytes = data.len(),
                    "OPFS write skipped: QuotaExceededError"
                );
                Ok(())
            } else {
                Err(OpfsError::Js(jsval.clone()))
            }
        }
        Err(e) => Err(e),
    }
}

/// Read the entire content of `name` in `dir` as `Vec<u8>`.
///
/// Returns `Err(OpfsError::FileNotFound)` when the file does not exist.
pub(crate) async fn opfs_read_all(
    dir: &FileSystemDirectoryHandle,
    name: &str,
) -> Result<Vec<u8>, OpfsError> {
    let fh_result = JsFuture::from(dir.get_file_handle(name)).await;
    let fh: FileSystemFileHandle = match fh_result {
        Ok(v) => v.dyn_into().map_err(OpfsError::Js)?,
        Err(jsval) => {
            let is_not_found = js_sys::Reflect::get(&jsval, &"name".into())
                .ok()
                .and_then(|v| v.as_string())
                .map(|n| n == "NotFoundError")
                .unwrap_or(false);
            if is_not_found {
                return Err(OpfsError::FileNotFound);
            }
            return Err(OpfsError::Js(jsval));
        }
    };

    let file: web_sys::File = JsFuture::from(fh.get_file()).await?.dyn_into()?;
    let ab: js_sys::ArrayBuffer = JsFuture::from(file.array_buffer()).await?.dyn_into()?;
    Ok(js_sys::Uint8Array::new(&ab).to_vec())
}

/// Return the current byte size of `name` in `dir`. Returns `0` when the
/// file does not exist.
pub(crate) async fn opfs_file_size(
    dir: &FileSystemDirectoryHandle,
    name: &str,
) -> Result<u64, OpfsError> {
    let fh_result = JsFuture::from(dir.get_file_handle(name)).await;
    let fh: FileSystemFileHandle = match fh_result {
        Ok(v) => v.dyn_into().map_err(OpfsError::Js)?,
        Err(jsval) => {
            let is_not_found = js_sys::Reflect::get(&jsval, &"name".into())
                .ok()
                .and_then(|v| v.as_string())
                .map(|n| n == "NotFoundError")
                .unwrap_or(false);
            if is_not_found {
                return Ok(0);
            }
            return Err(OpfsError::Js(jsval));
        }
    };

    let file: web_sys::File = JsFuture::from(fh.get_file()).await?.dyn_into()?;
    Ok(file.size() as u64)
}

/// Get the OPFS root `FileSystemDirectoryHandle`.
pub(crate) async fn opfs_root() -> Result<FileSystemDirectoryHandle, OpfsError> {
    let storage = web_sys::window()
        .ok_or(OpfsError::NoWindow)?
        .navigator()
        .storage();
    let root: FileSystemDirectoryHandle =
        JsFuture::from(storage.get_directory()).await?.dyn_into()?;
    Ok(root)
}
