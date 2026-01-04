use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    // In a real app, this would be persisted to a DB or file.
    // For now, in-memory.
    pub directories: Arc<Mutex<HashSet<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            directories: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}
