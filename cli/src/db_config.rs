use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Create {
    
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub addr: String,
    pub meili_host: String,
    pub meili_key: String,
}

