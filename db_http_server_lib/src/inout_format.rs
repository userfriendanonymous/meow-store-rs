use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum DecodeVal {
    Json(String),
    Bin(String),
}


#[derive(Clone, Copy, Debug)]
pub enum Value {
    Binary,
    Json
}

impl Value {
    pub fn encode_val_to_response<T: serde::Serialize + bincode::Encode>(&self, value: T) -> http::Response<hyper::body::Bytes> {
        let bytes = match self {
            Self::Binary => bincode::encode_to_vec(value, bincode::config::standard()).unwrap(),
            Self::Json => serde_json::to_vec(&value).unwrap()
        };
        http::Response::new(bytes.into())
    }

    pub fn decode_val_from_bytes<'de, T: serde::Deserialize<'de> + bincode::Decode>(&self, bytes: &'de [u8]) -> Result<T, DecodeVal> {
        match self {
            Self::Binary => bincode::decode_from_slice(bytes, bincode::config::standard())
                .map(|x| x.0)
                .map_err(|e| DecodeVal::Bin(e.to_string())),
            Self::Json => serde_json::from_slice(bytes).map_err(|e| DecodeVal::Json(e.to_string()))
        }
    }
}


impl FromStr for Value {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bin" => Ok(Self::Binary),
            "json" => Ok(Self::Json),
            _ => Err(())
        }
    }
}