use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Serialize, de::DeserializeOwned};
use std::{error::Error, fmt};
use wgpu::web_sys;

#[derive(Debug, Clone)]
pub(crate) struct Storage {
    handle: web_sys::Storage,
}

#[derive(Debug)]
pub(crate) enum StorageError {
    WindowUnavailable,
    LocalStorageUnavailable,
    LocalStorage(String),
    Serialization(serde_json::Error),
    Base64(base64::DecodeError),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowUnavailable => write!(f, "failed to get window"),
            Self::LocalStorageUnavailable => {
                write!(f, "failed to get local storage")
            }
            Self::LocalStorage(err) => write!(f, "browser storage error: {err}"),
            Self::Serialization(err) => write!(f, "serialization error: {err}"),
            Self::Base64(err) => write!(f, "base64 error: {err}"),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Serialization(err) => Some(err),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

impl From<base64::DecodeError> for StorageError {
    fn from(err: base64::DecodeError) -> Self {
        Self::Base64(err)
    }
}

type Result<T> = std::result::Result<T, StorageError>;

impl Storage {
    pub(crate) fn new() -> Result<Self> {
        let window = web_sys::window().ok_or(StorageError::WindowUnavailable)?;

        let handle = window
            .local_storage()
            .map_err(|err| StorageError::LocalStorage(format!("{err:?}")))?
            .ok_or(StorageError::LocalStorageUnavailable)?;

        Ok(Self { handle })
    }

    pub(crate) fn save<T>(&self, key: &str, value: &T) -> Result<String>
    where
        T: Serialize,
    {
        let json = serde_json::to_string(value)?;

        let base64 = BASE64_STANDARD.encode(json);

        self.handle
            .set_item(key, &base64)
            .map_err(|err| StorageError::LocalStorage(format!("{err:?}")))?;

        Ok(base64)
    }

    pub(crate) fn load<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let Some(base64) = self
            .handle
            .get_item(key)
            .map_err(|err| StorageError::LocalStorage(format!("{err:?}")))?
        else {
            return Ok(None);
        };

        let bytes = BASE64_STANDARD.decode(base64)?;

        let data = String::from_utf8_lossy(&bytes);

        Ok(Some(serde_json::from_str(&data)?))
    }
}
