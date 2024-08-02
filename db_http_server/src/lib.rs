

use std::borrow::Cow;
use std::convert::Infallible;
use std::path::Path;
use std::sync::Arc;
use std::{rc::Rc, str::FromStr};
pub use db::OpenMode;
use tokio::sync::{mpsc, RwLock};
use warp::Filter;
use warp::filters::path::param as warp_param;
use lib::{inout_format, InoutFormat};
use meilisearch_sdk::client::Client as MeiliClient;

mod users;
// mod tests;

// meilisearch --master-key="aSampleMasterKey"

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

pub async fn main(mode: OpenMode)
-> impl Filter<Extract = (impl warp::Reply,), Error = Infallible> + Clone + Send + Sync + Sized + 'static
{
    std::env::set_var("RUST_BACKTRACE", "1");

    let meili_client = MeiliClient::new("http://localhost:7700", Some("aSampleMasterKey")).unwrap();

    let (error_sender, mut error_receiver) = mpsc::channel(20);
    let _error_handle = tokio::spawn(async move {
        while let Some(err) = error_receiver.recv().await {
            println!("[INTERNAL ERROR]: {err:?}");
        }
    });

    let dir_path = Path::new("database_storage");
    if let OpenMode::New = mode {
        tokio::fs::create_dir_all(dir_path).await.unwrap();
    }

    let mut db = unsafe {
        db::Value::open(
            meili_client,
            dir_path,
            mode,
            error_sender,
        ).unwrap()
    };
    let db = Arc::new(RwLock::new(db));

    let filter = router(db.clone());
    filter
}
