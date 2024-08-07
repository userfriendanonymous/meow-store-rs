

use std::borrow::Cow;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::{rc::Rc, str::FromStr};
pub use db::OpenMode;
use tokio::sync::{mpsc, RwLock};
use warp::{serve, Filter};
use warp::filters::path::param as warp_param;
use lib::{inout_format, InoutFormat};
use meilisearch_sdk::client::Client as MeiliClient;

pub mod config;
mod users;

// mod tests;

// cd ~/Projects/meilidb && sudo meilisearch --master-key="aSampleMasterKey"

pub struct OptionAuthKey(Option<db::auth::Key>);

impl OptionAuthKey {
    pub fn as_ref(&self) -> Option<&db::auth::Key> {
        self.0.as_ref()
    }
}

impl FromStr for OptionAuthKey {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(db::auth::Key::from_str(s).ok()))
    }
}

fn auth_key_filter()
-> impl Filter<Extract = (OptionAuthKey,), Error = Infallible> + Clone {
    warp::header::<OptionAuthKey>("x-auth-key")
        .or(warp::any().map(|| OptionAuthKey(None)))
        .unify()
}

fn router(db: Arc<RwLock<db::Value>>)
-> impl Filter<Extract = (impl warp::Reply,), Error = Infallible> + Clone + Send + Sync + Sized + 'static {
        warp::path("users").and(
            users::filter(db.clone())
        )
        .or(
            warp::any()
            .map(|| "Hello world!")
        )
}

// pub async fn main(mode: OpenMode)
// -> impl Filter<Extract = (impl warp::Reply,), Error = Infallible> + Clone + Send + Sync + Sized + 'static
// {
//     std::env::set_var("RUST_BACKTRACE", "1");

//     let meili_client = MeiliClient::new("http://localhost:7700", Some("aSampleMasterKey")).unwrap();

//     let (error_sender, mut error_receiver) = mpsc::channel(20);
//     let _error_handle = tokio::spawn(async move {
//         while let Some(err) = error_receiver.recv().await {
//             println!("[INTERNAL ERROR]: {err:?}");
//         }
//     });

//     let dir_path = Path::new("database_storage");
//     if let OpenMode::New = mode {
//         tokio::fs::create_dir_all(dir_path).await.unwrap();
//     }

//     let mut db = unsafe {
//         db::Value::open(
//             meili_client,
//             dir_path,
//             mode,

//             error_sender,
//         ).unwrap()
//     };
//     let db = Arc::new(RwLock::new(db));

//     let filter = router(db.clone());
//     filter
// }

#[derive(Clone)]
pub struct Init {
    pub db: Arc<RwLock<db::Value>>,
    pub addr: SocketAddr,
}

impl Init {
    pub async fn run(self) {
        let filter = router(self.db);
        warp::serve(filter)
            .run(self.addr)
            .await
    }
}

pub async fn init_with_config(config: config::Run) -> Init {
    let meili_client = MeiliClient::new(config.meili_addr, Some(config.meili_key)).unwrap();

    let (error_sender, mut error_receiver) = mpsc::channel(20);
    let _error_handle = tokio::spawn(async move {
        while let Some(err) = error_receiver.recv().await {
            println!("[INTERNAL ERROR]: {err:?}");
        }
    });

    tokio::fs::create_dir_all(&config.db_path).await.unwrap();

    let mut db = unsafe {
        db::Value::open(
            meili_client,
            &config.db_path,
            config.mode,
            config.db,
            error_sender,
        ).unwrap()
    };
    let db = Arc::new(RwLock::new(db));
    Init {
        db,
        addr: config.addr
    }
}
