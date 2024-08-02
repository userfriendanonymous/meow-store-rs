use warp::Filter;
use warp::Reply;

#[shuttle_runtime::main]
async fn warp() -> shuttle_warp::ShuttleWarp<(impl Reply,)> {
    let db = db_http_server::create_ctx().await;
    let route = db_http_server::router(db);
    Ok(route.boxed().into())
}
