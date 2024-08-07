
use http_input::Instance as HttpInput;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum GetByNameError {
    Get(db::user::GetByNameError)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum SearchError {
    Search(db::user::SearchError)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum WriteError {
    DecodeInput(super::inout_format::DecodeVal),
    Add(db::user::AddError),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum RemoveByNameError {
    Remove(db::user::RemoveByNameError),
}

pub type GetByNameOutput = Result<db::user::Value<'static>, GetByNameError>;
pub type WriteOutput = Result<bool, WriteError>;
pub type RemoveByNameOutput = Result<bool, RemoveByNameError>;

// pub struct WriteInput<'a>(String, db::User<'a>);

// impl<'a> HttpInput for WriteInput<'a> {
//     type Output = WriteOutput;
//     fn into_request(self) -> http::Result<Request<http_input::Bytes>> {
//         http::Request::get(format!("https://{}/users/{}", self.0, &self.1))
//             .body(Bytes::new())
//     }
// }