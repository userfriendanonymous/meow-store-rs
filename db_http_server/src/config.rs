use std::{net::SocketAddr, path::PathBuf};

use db::OpenMode;

#[derive(Clone, Debug)]
pub struct Run {
    pub mode: OpenMode,
    pub db_path: PathBuf,
    pub addr: SocketAddr,
    pub meili_addr: String,
    pub meili_key: String,
    pub db: db::config::Root,
}
