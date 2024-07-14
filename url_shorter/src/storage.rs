use dashmap::DashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("key already exists")]
    AlreadyExists,
}

pub trait Storage {
    fn get(&self, key: String) -> Option<String>;
    fn set(&self, key: String, value: String) -> Result<(), StorageError>;
}

pub struct InMemoryStorage {
    data: DashMap<String, String>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: DashMap::new(),
        }
    }
}

impl Storage for InMemoryStorage {
    fn get(&self, key: String) -> Option<String> {
        self.data.get(&key).map(|v| v.value().to_owned())
    }

    fn set(&self, key: String, value: String) -> Result<(), StorageError> {
        if self.data.contains_key(&key) {
            return Err(StorageError::AlreadyExists);
        }
        self.data.insert(key, value);
        Ok(())
    }
}
