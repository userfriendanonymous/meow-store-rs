use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub db_url: String,
    pub db_auth_key: Option<db::auth::Key>,
    pub initial_user: String,
}