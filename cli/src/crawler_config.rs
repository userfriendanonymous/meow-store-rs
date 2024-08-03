use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub db_url: String,
    pub initial_user: String,
}