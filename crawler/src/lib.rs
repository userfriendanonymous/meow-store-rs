use std::{borrow::Cow, collections::VecDeque, time::Duration};
use reqwest::header::HeaderValue;
use rs2s::{input::ItemsRange, Username};
use http_input::{Bytes, Instance as HttpInput};
pub mod config;

pub async fn run_with_config(config: config::Run) {
    let client = reqwest::Client::new();
    let mut state = State::new(client, &config.db_url, config.db_auth_key);
    state.request_queue.users.push(rs2s::input::User(Username::new(config.initial_user)));
    loop {
        state.request_respond().await.unwrap();
    }
}

#[derive(Debug)]
pub enum RequestError {
    HttpSend(http_input::reqwest::SendError)
}

#[derive(Debug)]
pub enum Error {
    ParseUsername,
}

#[derive(Debug)]
pub enum RespondError {
    Http(reqwest::Error),
    Decode(bincode::error::DecodeError),
}

#[derive(Debug)]
pub enum RequestRespondError {
    Request(RequestError),
    Respond(RespondError)
}

pub struct State {
    request_queue: RequestQueue,
    //request_marks: RequestMarks,
    http_client: reqwest::Client,
    response_queue: ResponseQueue,
    error_queue: Vec<Error>,
    db_url: reqwest::Url,
    bincode_config: bincode::config::Configuration,
    auth_key: Option<db::auth::Key>,
}

impl State {
    pub fn new(http_client: reqwest::Client, db_url: impl reqwest::IntoUrl, auth_key: Option<db::auth::Key>) -> Self {
        Self {
            request_queue: RequestQueue::default(),
            response_queue: ResponseQueue::default(),
            error_queue: Vec::new(),
            db_url: db_url.into_url().unwrap(),
            http_client,
            bincode_config: bincode::config::standard(),
            auth_key,
        }
    }

    async fn http_send<I: HttpInput>(&self, input: I) -> Result<I::Output, http_input::reqwest::SendError> {
        tokio::time::sleep(Duration::from_millis(200)).await;
        http_input::reqwest::send(&self.http_client, input).await
    }

    pub async fn request_respond(&mut self) -> Result<(), RequestRespondError> {
        self.request_all().await.map_err(RequestRespondError::Request)?;
        self.respond_all().await.map_err(RequestRespondError::Respond)?;
        Ok(())
    }

    pub async fn request_user(&mut self) -> Result<(), RequestError> {
        type E = RequestError;

        if let Some(request) = self.request_queue.users.pop() {
            let out = self.http_send(request).await.map_err(E::HttpSend)?;

            if let Ok(out) = out {
                println!("Name: {}", &out.name);
                let Ok(db_name) = out.name.parse::<db::Username>() else {
                    self.error_queue.push(Error::ParseUsername);
                    return Ok(());
                };
                let name = rs2s::Username::new(&out.name);

                let mut req = rs2s::input::user::Projects(name, rs2s::input::ItemsRange { offset: 0, limit: 40 });
                let (mut loves, mut favorites, mut remixes, mut views) = (0, 0, 0, 0);

                loop {
                    let res = self.http_send(req.clone()).await.map_err(E::HttpSend)?.unwrap();
                    for project in &res {
                        loves += project.stats.loves;
                        favorites += project.stats.favorites;
                        remixes += project.stats.remixes;
                        views += project.stats.views;
                    }

                    if res.len() < 40 {
                        break;
                    }
                    req.1.offset += 40;
                    if req.1.offset > 500 {
                        return Ok(());
                    }

                    println!("project_offset: {}", req.1.offset);
                }

                self.response_queue.users.push(db::User {
                    name: db_name,
                    id: out.id,
                    scratch_team: out.scratch_team,
                    status: Cow::Owned(out.profile.status),
                    bio: Cow::Owned(out.profile.bio),
                    loves,
                    favorites,
                    views,
                    remixes,
                });
            }
        }
        Ok(())
    }

    pub async fn request_user_followers(&mut self) -> Result<(), RequestError> {
        type E = RequestError;
        if let Some(req) = self.request_queue.users_followers.pop() {
            let out = self.http_send(req).await.map_err(E::HttpSend)?;
            match out {
                Ok(out) => {
                    for user in out {
                        let name = Username::new(user.name);
                        self.request_queue.users.push(rs2s::input::User(name.clone()));
                    }
                },
                Err(e) => {}
            }
        }
        Ok(())
    }

    pub async fn request_all(&mut self) -> Result<(), RequestError> {
        self.request_user().await?;
        self.request_user_followers().await?;
        Ok(())
    }

    pub async fn respond_user(&mut self) -> Result<(), RespondError> {
        if let Some(user) = self.response_queue.users.pop() {
            let name = Username::new(user.name.to_string());

            let res = self.http_client
                .post(self.db_url.join("/users/write/bin/bin").unwrap())
                .body(bincode::encode_to_vec(user, self.bincode_config).unwrap())
                .header(
                    "x-auth-key",
                    self.auth_key.as_ref().map(|x| HeaderValue::from_bytes(x.as_bytes()).unwrap())
                        .unwrap_or(HeaderValue::from_static(""))
                )
                .send()
                .await
                .map_err(RespondError::Http)?;
            let bytes = res.bytes().await.map_err(RespondError::Http)?;
            let res = bincode::decode_from_slice::<db_http_server::users::WriteOutput, _>(bytes.as_ref(), self.bincode_config)
                .map_err(RespondError::Decode)?.0;
            println!("Db response: {:?}", &res);

            if res.is_ok_and(|i| !i) {
                self.request_queue.users_followers.push(rs2s::input::user::Followers(name, rs2s::input::ItemsRange { offset: 0, limit: 40 }));
            }
        }
        Ok(())
    }

    pub async fn respond_all(&mut self) -> Result<(), RespondError> {
        self.respond_user().await?;
        Ok(())
    }
}

#[derive(Default)]
pub struct ResponseQueue {
    pub users: Vec<db::User<'static>>
}

#[derive(Default)]
pub struct RequestQueue {
    pub users: Vec<rs2s::input::User<'static>>,
    pub users_followers: Vec<rs2s::input::user::Followers<'static>>,

}

// pub struct BoundQueue<T> {
//     max: usize,
//     vec: VecDeque<T>,
// }

// impl<T> BoundQueue<T> {
//     pub fn new(max: usize) -> Self {
//         Self {
//             max,
//             vec: VecDeque::new()
//         }
//     }

//     pub fn push(&mut self, value: T) {
//         self.vec.push_front(value)
//     }
// }

// pub struct RequestMarks {
//     pub users: VecDeque<>
// }