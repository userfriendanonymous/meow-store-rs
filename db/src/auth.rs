use core::str;
use std::{array, fmt::Display, str::FromStr};

use bindb::storage::OpenMode;
use rand::{prelude::Distribution, Rng};
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};

use crate::{BindbErrorKind, BindbErrorOp};

pub const KEY_LEN: usize = 16;

binbuf::fixed! {
    #[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
    pub struct Key([u8; KEY_LEN]);
    buf! { pub struct KeyBuf<P>(Key, P); }
    impl I for Key { type Buf<P> = KeyBuf<P>; }
    impl Code for Key {}
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
        String::deserialize(deserializer)?.parse()
            .map_err(|_| serde::de::Error::custom("invalid"))
    }
}

impl Key {
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.0).unwrap()
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(str::from_utf8(&self.0).unwrap())
    }
}

impl FromStr for Key {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        bytes.try_into()
            .map(|arr: &[u8; KEY_LEN]| Key(*arr))
            .map_err(|_| ())
    }
}

binbuf::fixed! {
    #[derive(Clone, Debug)]
    pub struct Desc {
        pub write: bool,
        pub remove: bool,
        pub read: bool,
    }
    buf! { pub struct DescBuf<P>(Desc, P); }
    impl I for Desc { type Buf<P> = DescBuf<P>; }
    impl Code for Desc {}
}

impl Desc {
    pub fn new_all_false() -> Self {
        Self {
            read: false,
            write: false,
            remove: false,
        }
    }

    pub fn is_op_allowed(&self, op: Op) -> bool {
        match op {
            Op::Read => self.read,
            Op::Write => self.write,
            Op::Remove => self.remove,
        }
    }
}

pub type Store = bindb::storage::BinaryTree<u16, Key, Desc>;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum GenError {
    Internal,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum EnsureAuthError {
    Required,
    Invalid,
    NotAllowed,
}

pub enum Op {
    Read,
    Write,
    Remove,
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => f.write_str("read"),
            Self::Write => f.write_str("write"),
            Self::Remove => f.write_str("remove"),
        }
    }
}

impl super::Value {
    pub async fn gen_auth(&mut self, desc: &Desc) -> Result<Key, GenError> {
        let mut key = [0; KEY_LEN];

        let key_vec = &rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(KEY_LEN)
            .collect::<Vec<_>>();
        key.copy_from_slice(&key_vec);

        match self.auth.add(&Key(key), desc) {
            Ok(already_exists) => {
                if already_exists {
                    Err(GenError::Internal)?
                }
            },
            Err(err) => {
                self.send_bindb_error(BindbErrorOp::GenAuth, BindbErrorKind::BinaryTreeAdd(err)).await;
                Err(GenError::Internal)?
            }
        }
        Ok(Key(key))
    }

    pub(super) fn auth_desc_by_key(&self, key: &Key) -> Option<Desc> {
        self.auth.get(key)
    }

    pub(super) fn ensure_auth(&self, op: Op, key: Option<&Key>) -> Result<(), EnsureAuthError> {
        let require = match op {
            Op::Read => self.config.require_auth.read,
            Op::Write => self.config.require_auth.write,
            Op::Remove => self.config.require_auth.remove,
        };
        if require {
            if !self.auth_desc_by_key(key.ok_or(EnsureAuthError::Required)?)
                .ok_or(EnsureAuthError::Invalid)?
                .is_op_allowed(op)
            {
                Err(EnsureAuthError::NotAllowed)?
            }
        }
        Ok(())
    }
}
