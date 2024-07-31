use std::borrow::Cow;
use binbuf::{BytesPtrConst, BytesPtr, Dynamic, Fixed};
use crate::{BindbError, BindbErrorKind, BindbErrorOp, InternalError};
use binbuf::impls::ArbNum;
use super::Username as Name;
use serde::{Serialize, Deserialize};
use bincode::Decode;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeiliDoc {
    pub id: u64,
    pub status: String,
    pub bio: String,
}

// #[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
// pub struct Stats {
    
// }

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Value<'a> {
    pub name: Name,
    pub id: u64,
    pub scratch_team: bool,
    pub status: Cow<'a, str>,
    pub bio: Cow<'a, str>,
    // Statistics
    pub loves: u32,
    pub favorites: u32,
    pub views: u32,
    pub remixes: u32,
}

impl From<DbValue> for Value<'static> {
    fn from(value: DbValue) -> Self {
        Self {
            name: value.fixed_data.name,
            id: value.fixed_data.id,
            scratch_team: value.fixed_data.scratch_team,
            status: Cow::Owned(value.status),
            bio: Cow::Owned(value.bio),
            loves: value.fixed_data.loves,
            favorites: value.fixed_data.favorites,
            views: value.fixed_data.views,
            remixes: value.fixed_data.remixes,
        }
    }
}

impl<'a> Value<'a> {
    pub unsafe fn to_db_value(self) -> DbValue {
        DbValue {
            fixed_data: FixedData {
                name: self.name.clone(),
                id: self.id,
                scratch_team: self.scratch_team,
                loves: self.loves,
                favorites: self.favorites,
                remixes: self.remixes,
                views: self.views,
            },
            status: self.status.into_owned(),
            bio: self.bio.into_owned(),
        }
    }
}

binbuf::dynamic! {
    pub struct DbValue {
        pub fixed_data: FixedData,
        pub status: String,
        pub bio: String,
    }

    buf! { pub struct DbValueBuf<P>(DbValue, P); }

    impl I for DbValue {
        type Buf<P> = DbValueBuf<P>;
    }

    impl Code for DbValue {}
}

binbuf::fixed! {
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct FixedData {
        #[lens(buf_name)]
        pub name: Name,
        pub id: u64,
        pub scratch_team: bool,
        pub loves: u32,
        pub favorites: u32,
        pub views: u32,
        pub remixes: u32,
    }

    buf! { pub struct FixedDataBuf<P>(FixedData, P); }

    impl I for FixedData {
        type Buf<P> = FixedDataBuf<P>;
    }
    impl Code for FixedData {}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum AddError {
    Internal,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum SearchError {
    Internal,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum RemoveByNameError {
    Internal,
}

impl super::Value {
    unsafe fn add_user_raw<'a>(&mut self, data: Value<'a>) -> Result<u64, bindb::storage::indexed_dynamic::AddError> {
        self.users.add(&data.to_db_value())
    }

    // Returns true if already exists.
    pub async fn add_user<'a>(&mut self, data: Value<'a>) -> Result<bool, AddError> {
        let searched = self.users_name_index.search(&data.name);
        match searched.find() {
            Ok(_) => Ok(true),
            Err(searched) => {
                let name = data.name.clone();
                let id = match unsafe { self.add_user_raw(data.clone()) } {
                    Ok(id) => id,
                    Err(e) => {
                        self.send_bindb_error(BindbErrorOp::AddUser, BindbErrorKind::IndexedDynamicAdd(e)).await;
                        Err(AddError::Internal)?
                    }
                };

                if let Err(e) = unsafe {
                    self.users_name_index.add_searched(&searched, &name, &ArbNum::new(id))
                } {
                    self.send_bindb_error(BindbErrorOp::AddUser, BindbErrorKind::BinaryTreeAdd(e)).await;
                    Err(AddError::Internal)?
                }

                let meili_doc = MeiliDoc {
                    id,
                    status: data.status.to_string(),
                    bio: data.bio.to_string()
                };

                let info = match self.meili_client
                    .index("users")
                    .add_documents(&[meili_doc], None)
                    .await
                {
                    Ok(info) => info,
                    Err(e) => {
                        self.send_meili_error(e).await;
                        Err(AddError::Internal)?
                    }
                };
                println!("Task info: {:?}", info);

                Ok(false)
            }
        }
    }

    pub fn user_by_name(&self, name: &Name) -> Option<Value<'static>> {
        self.users_name_index.get(name).map(|id| {
            self.users.get(id.unwrap()).into()
        })
    }

    pub async fn search_users<'a, 'b>(&'a self, query: &'b str) -> Result<Vec<Value<'static>>, SearchError> {
        let res = self.meili_client.index("users")
            .search()
            .with_query(query)
            .execute::<MeiliDoc>()
            .await;

        let res = match res {
            Ok(res) => res,
            Err(e) => {
                self.send_meili_error(e).await;
                Err(SearchError::Internal)?
            }
        };

        let mut hits = Vec::with_capacity(res.hits.len());
        for hit in res.hits {
            hits.push(Value::from(self.users.get(hit.result.id)));
        }
        Ok(hits)
    }

    pub async fn remove_user_by_name(&mut self, name: &Name) -> Result<bool, RemoveByNameError> {
        let searched = self.users_name_index.search(name);
        match searched.find() {
            Ok(searched) => {
                let id = unsafe { self.users_name_index.get_searched(&searched).unwrap() };
                if let Err(e) = unsafe { self.users.remove(id) } {
                    self.send_bindb_error(BindbErrorOp::RemoveUserByName, BindbErrorKind::IndexedDynamicRemove(e)).await;
                    Err(RemoveByNameError::Internal)?
                }
                if let Err(e) = unsafe { self.users_name_index.remove_searched(&searched) } {
                    self.send_bindb_error(BindbErrorOp::RemoveUserByName, BindbErrorKind::BinaryTreeRemove(e)).await;
                    Err(RemoveByNameError::Internal)?
                }
                Ok(false)
            },
            Err(_) => Ok(true)
        }
    }
}