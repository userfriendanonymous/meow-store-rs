use std::{str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use warp::{filters::path::param as warp_param, reject::Rejection, Filter};
use crate::{auth_key_filter, inout_format, InoutFormat, OptionAuthKey};
use db::Username as DbUsername;
use lib::users::*;

pub fn filter(db: Arc<RwLock<db::Value>>)
-> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    warp::path!("get_by_name" / DbUsername / InoutFormat)
        .and(warp::get())
        .and(auth_key_filter())
        .then({
            let db = db.clone();
            move |name, out_format: InoutFormat, auth_key: OptionAuthKey| {
                let db = (&db).clone();
                async move {
                    let out = db.read().await.user_by_name(auth_key.as_ref(), &name)
                        .map_err(GetByNameError::Get);
                    out_format.encode_val_to_response(&out)
                }
            }
        })
    .or(
        warp::path!("search" / String / InoutFormat)
            .and(warp::get())
            .and(auth_key_filter())
            .then({
                let db = db.clone();
                move |query: String, out_format: InoutFormat, auth_key: OptionAuthKey| {
                    let db = db.clone();
                    async move {
                        let out = db.read().await.search_users(auth_key.as_ref(), &query)
                            .await
                            .map_err(SearchError::Search);
                        out_format.encode_val_to_response(&out)
                    }
                }
            })
    )
    .or(
        warp::path!("write" / InoutFormat / InoutFormat)
            .and(warp::post())
            .and(warp::body::bytes())
            .and(auth_key_filter())
            .then({
                let db = db.clone();
                move |in_format: InoutFormat, out_format: InoutFormat, body: hyper::body::Bytes, auth_key: OptionAuthKey| {
                    let db = (&db).clone();
                    async move {
                        let out = match in_format.decode_val_from_bytes(&body) {
                            Ok(data) => {
                                    println!("{:?}", &data);
                                    db.write().await.add_user(auth_key.as_ref(), data).await
                                        .map_err(WriteError::Add)
                            },
                            Err(e) => Err(WriteError::DecodeInput(e))
                        };
                        out_format.encode_val_to_response(&out)
                    }
                }
            })
    )
    .or(
        warp::path!("remove_by_name" / DbUsername / InoutFormat)
            .and(auth_key_filter())
            .then({
                let db = db.clone();
                move |name, out_format: InoutFormat, auth_key: OptionAuthKey| {
                    let db = (&db).clone();
                    async move {
                        let out = db.write().await.remove_user_by_name(auth_key.as_ref(), &name)
                            .await
                            .map_err(RemoveByNameError::Remove);
                        out_format.encode_val_to_response(&out)
                    }
                }
            })
    )
}

// users/get/json/by_name/griffpatch
// users/write/json/json
// users/get_by_name/griffpatch/json
// users/get_by_id/5432/json
