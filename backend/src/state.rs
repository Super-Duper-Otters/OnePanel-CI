use crate::db::DbPool;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    // In a real app, this would be persisted to a DB or file.
    // For now, in-memory.
    pub directories: Arc<Mutex<HashSet<String>>>,
    pub db: Arc<DbPool>,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self {
            directories: Arc::new(Mutex::new(HashSet::new())),
            db: Arc::new(db),
        }
    }
}
