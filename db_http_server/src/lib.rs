

use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;
use std::{rc::Rc, str::FromStr};
use tokio::sync::{mpsc, RwLock};
use warp::Filter;
use warp::filters::path::param as warp_param;
use lib::{inout_format, InoutFormat};
use meilisearch_sdk::client::Client as MeiliClient;

mod users;
// mod tests;

// meilisearch --master-key="aSampleMasterKey"

pub fn router(db: Arc<RwLock<db::Value>>)
-> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + Send + Sync + 'static {
    warp::path("users").and(
        users::filter(db.clone())
    )
    .or(
        warp::any()
        .map(|| "Hello world!")
    )
}

pub async fn create_ctx() -> Arc<RwLock<db::Value>> {
    let meili_client = MeiliClient::new("http://localhost:7700", Some("aSampleMasterKey")).unwrap();

    let (error_sender, mut error_receiver) = mpsc::channel(20);

    let error_handle = tokio::spawn(async move {
        while let Some(err) = error_receiver.recv().await {
            println!("[INTERNAL ERROR]: {err:?}");
        }
    });

    let mut db = unsafe {
        db::Value::open(
            meili_client,
            Path::new("database_storage"),
            db::OpenMode::Existing,
            error_sender,
        ).unwrap()
    };

    // db.add_user(db::user::Value {
    //     name: "griffpatch".parse().unwrap(),
    //     id: 104492,
    //     scratch_team: false,
    //     status: Cow::Borrowed("Some status..."),
    //     bio: Cow::Borrowed("Some cool bio!"),
    // }).await.unwrap();

    let db = Arc::new(RwLock::new(db));
    db
}

async fn r() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let meili_client = MeiliClient::new("http://localhost:7700", Some("aSampleMasterKey")).unwrap();

    let (error_sender, mut error_receiver) = mpsc::channel(20);

    let error_handle = tokio::spawn(async move {
        while let Some(err) = error_receiver.recv().await {
            println!("[INTERNAL ERROR]: {err:?}");
        }
    });

    let mut db = unsafe {
        db::Value::open(
            meili_client,
            Path::new("database_storage"),
            db::OpenMode::Existing,
            error_sender,
        ).unwrap()
    };

    // db.add_user(db::user::Value {
    //     name: "griffpatch".parse().unwrap(),
    //     id: 104492,
    //     scratch_team: false,
    //     status: Cow::Borrowed("Some status..."),
    //     bio: Cow::Borrowed("Some cool bio!"),
    // }).await.unwrap();

    let db = Arc::new(RwLock::new(db));

    let filter = router(db.clone());
        
    warp::serve(filter)
        .run(([127, 0, 0, 1], 3030))
        .await;

    error_handle.await.unwrap();
}
