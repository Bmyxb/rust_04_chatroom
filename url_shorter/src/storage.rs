use dashmap::DashMap;

pub trait Storage {
    fn get(&self, key: String) -> Option<String>;
    fn set(&self, key: String, value: String);
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
        self.data.get(&key).map(|v| v.value().clone())
    }
    fn set(&self, key: String, value: String) {
        self.data.insert(key, value);
    }
}
