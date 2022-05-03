use crate::StoreImpl;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct SledStore {
    db: sled::Db,
}

pub use SledStore as InnerStore;

#[derive(thiserror::Error, Debug)]
pub enum GetError {
    #[error("Sled error")]
    Sled(#[from] sled::Error),
    #[error("MessagePack deserialization error")]
    MessagePack(#[from] rmp_serde::decode::Error),
    #[error("No value found for the given key")]
    NotFound,
}

#[derive(thiserror::Error, Debug)]
pub enum SetError {
    #[error("Sled error")]
    Sled(#[from] sled::Error),
    #[error("MessagePack serialization error")]
    MessagePack(#[from] rmp_serde::encode::Error),
}

impl Default for SledStore {
    // todo: maybe consider not exposing this, so people are not tempted
    // to try to manually initialize stores?
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let db = sled::open("bevy_pkv.sled").expect("Failed to init key value store");
            Self { db }
        }
    }
}

impl StoreImpl for SledStore {
    type GetError = GetError;
    type SetError = SetError;

    /// Serialize and store the value
    fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Self::SetError> {
        let mut serializer = rmp_serde::Serializer::new(Vec::new()).with_struct_map();
        value.serialize(&mut serializer)?;
        self.db.insert(key, serializer.into_inner())?;
        Ok(())
    }

    /// More or less the same as set::<String>, but can take a &str
    fn set_string(&mut self, key: &str, value: &str) -> Result<(), Self::SetError> {
        let bytes = rmp_serde::to_vec(value)?;
        self.db.insert(key, bytes)?;
        Ok(())
    }

    /// Get the value for the given key
    /// returns Err(GetError::NotFound) if the key does not exist in the key value store.
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, Self::GetError> {
        let bytes = self.db.get(key)?.ok_or(Self::GetError::NotFound)?;
        let value = rmp_serde::from_slice(&bytes)?;
        Ok(value)
    }
}
