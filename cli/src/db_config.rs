use serde::{Serialize, Deserialize};
pub use db::config::RequireAuth;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Create {
    
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub addr: String,
    pub meili_host: String,
    pub meili_key: String,
    pub require_auth: RequireAuth,
}
