use db_http_server::OpenMode;
use warp::Filter;
use warp::Reply;

#[shuttle_runtime::main]
async fn warp() -> shuttle_warp::ShuttleWarp<(impl Reply,)> {
    let route = db_http_server::main(OpenMode::New).await;
    Ok(route.boxed().into())
}
