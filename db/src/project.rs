use bitflags::bitflags;
use crate::{auth, BindbErrorKind, BindbErrorOp};
use binbuf::impls::ArbNum;
use super::Username;
use binbuf::impls::dynamic::StringCLL;

// binbuf::fixed! {
//     #[derive(Clone, Copy, Debug)]
//     pub enum Visibility {
//         Visible,
//     }
//     buf! { pub struct VisibilityBuf<P>(Visibility, P); }
//     impl I for Visibility { type Buf<P> = VisibilityBuf<P>; }
//     impl Code for Visibility {}
// }

bitflags! {
    pub struct Flags: u8 {
        const PUBLIC = 1;
        const COMMENTS_ALLOWED = 1 << 1;
        const IS_PUBLISHED = 1 << 2;
        const AUTHOR_SCRATCH_TEAM = 1 << 3;
    }
}

binbuf::dynamic! {
    pub struct DbRepr {
        pub flags: u8,
        pub id: u64,
        // pub visibility: Visibility,
        // pub public: bool,
        // pub comments_allowed: bool,
        // pub is_published: bool,
        pub author_id: u64,
        pub author_name: Username,
        // pub author_scratch_team: bool,
        pub created: i64,
        pub modified: i64,
        pub shared: i64,
        pub title: StringCLL<2>,
        pub description: StringCLL<2>,
        pub instructions: StringCLL<2>,
    }
    buf! { pub struct DbReprBuf<P>(DbRepr, P); }
    impl I for DbRepr { type Buf<P> = DbReprBuf<P>; }
    impl Code for DbRepr {}
}

impl From<DbRepr> for Value {
    fn from(value: DbRepr) -> Self {
        let flags = Flags::from_bits_retain(value.flags);
        Self {
            id: value.id,
            public: flags.contains(Flags::PUBLIC),
            is_published: flags.contains(Flags::IS_PUBLISHED),
            comments_allowed: flags.contains(Flags::COMMENTS_ALLOWED),
            author_scratch_team: flags.contains(Flags::AUTHOR_SCRATCH_TEAM),
            author_id: value.author_id,
            author_name: value.author_name,
            created: value.created,
            modified: value.modified,
            shared: value.shared,
            title: value.title.into(),
            description: value.description.into(),
            instructions: value.instructions.into(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct Value {
    pub id: u64,
    pub public: bool,
    pub comments_allowed: bool,
    pub is_published: bool,
    pub author_id: u64,
    pub author_name: Username,
    pub author_scratch_team: bool,
    pub created: i64,
    pub modified: i64,
    pub shared: i64,
    pub title: String,
    pub description: String,
    pub instructions: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum ToDbReprError {
    TitleTooLong,
    DescriptionTooLong,
    InstructionsTooLong,
}

impl Value {
    pub fn to_db_repr(self) -> Result<DbRepr, ToDbReprError> {
        let mut flags = Flags::empty();
        if self.public { flags |= Flags::PUBLIC; }
        if self.is_published { flags |= Flags::IS_PUBLISHED; }
        if self.comments_allowed { flags |= Flags::COMMENTS_ALLOWED; }
        if self.author_scratch_team { flags |= Flags::AUTHOR_SCRATCH_TEAM; }
        Ok(DbRepr {
            flags: flags.bits(),
            id: self.id,
            author_id: self.author_id,
            author_name: self.author_name,
            created: self.created,
            modified: self.modified,
            shared: self.shared,
            title: StringCLL::try_from_string(self.title).ok_or(ToDbReprError::TitleTooLong)?,
            description: StringCLL::try_from_string(self.description).ok_or(ToDbReprError::DescriptionTooLong)?,
            instructions: StringCLL::try_from_string(self.instructions).ok_or(ToDbReprError::InstructionsTooLong)?,
        })
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MeiliDoc {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub instructions: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum GetByIdError {
    Auth(auth::EnsureAuthError),
    NotFound,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub enum AddError {
    Internal,
    Auth(auth::EnsureAuthError),
    BadInput(ToDbReprError),
}

impl super::Value {
    pub fn project_by_id(&self, auth_key: Option<&auth::Key>, id: &u64) -> Result<Value, GetByIdError> {
        self.ensure_auth(auth::Op::Read, auth_key).map_err(GetByIdError::Auth)?;
        self.projects_id_index.get(id)
            .map(|id| {
                self.projects.get(id.get()).into()
            })
            .ok_or(GetByIdError::NotFound)
    }

    pub async fn add_project(&mut self, auth_key: Option<&auth::Key>, value: Value) -> Result<bool, AddError> {
        self.ensure_auth(auth::Op::Write, auth_key).map_err(AddError::Auth)?;
        let searched = self.projects_id_index.search(&value.id);
        match searched.find() {
            Ok(_) => Ok(true),
            Err(searched) => {
                let value_id = value.id;

                let db_repr = value.clone().to_db_repr().map_err(AddError::BadInput)?;
                let id = match unsafe { self.projects.add(&db_repr) } {
                    Ok(id) => id,
                    Err(e) => {
                        self.send_bindb_error(BindbErrorOp::AddProject, BindbErrorKind::IndexedDynamicAdd(e)).await;
                        Err(AddError::Internal)?
                    }
                };

                let meili_doc = MeiliDoc {
                    id,
                    title: value.title,
                    description: value.description,
                    instructions: value.instructions,
                };

                if let Err(e) = unsafe {
                    self.projects_id_index.add_searched(&searched, &value_id, &ArbNum::new(id))
                } {
                    self.send_bindb_error(BindbErrorOp::AddProject, BindbErrorKind::BinaryTreeAdd(e)).await;
                    Err(AddError::Internal)?
                }
                
                let info = match self.meili_client
                    .index("projects")
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
}