use crate::db::DbPool;
use std::sync::Arc;

use std::collections::HashMap;
use tokio::sync::{mpsc::UnboundedSender, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
    pub mcp_sessions: Arc<RwLock<HashMap<String, UnboundedSender<String>>>>,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self {
            db: Arc::new(db),
            mcp_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
