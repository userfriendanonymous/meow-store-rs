use bindb::storage::OpenMode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Root {
    pub require_auth: RequireAuth,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequireAuth {
    pub read: bool,
    pub write: bool,
    pub remove: bool,
}