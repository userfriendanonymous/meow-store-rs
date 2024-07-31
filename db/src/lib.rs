use std::{borrow::Cow, fs::File, path::{Path, PathBuf}, str::FromStr};
use meilisearch_sdk::client::Client as MeiliClient;
use binbuf::{BytesPtr, bytes_ptr, impls::{ArbNum, arb_num}};
pub use bindb::storage::OpenMode;
pub use username::Value as Username;
pub use user::Value as User;
use tokio::sync::mpsc;
// pub use country::Value as Country;

pub mod username;
// pub mod country;
pub mod user;

#[derive(Debug)]
pub enum InternalError {
    Meili(meilisearch_sdk::errors::Error),
    Bindb(BindbError),
}

impl InternalError {
    pub fn bindb(op: BindbErrorOp, kind: BindbErrorKind) -> Self {
        Self::Bindb(BindbError::new(op, kind))
    }
}

#[derive(Debug)]
pub struct BindbError {
    pub op: BindbErrorOp,
    pub kind: BindbErrorKind,
}

impl BindbError {
    pub fn new(op: BindbErrorOp, kind: BindbErrorKind) -> Self {
        Self { op, kind }
    }
}

#[derive(Debug)]
pub enum BindbErrorOp {
    AddUser,
    UserByName,
    SearchUsers,
    RemoveUserByName,
}

#[derive(Debug)]
pub enum BindbErrorKind {
    IndexedDynamicAdd(bindb::storage::indexed_dynamic::AddError),
    BinaryTreeAdd(bindb::storage::binary_tree::AddError),
    BinaryTreeRemove(bindb::storage::binary_tree::RemoveError),
    IndexedDynamicRemove(bindb::storage::indexed_dynamic::RemoveError),
}

// region: OpenError
#[derive(Debug)]
pub enum OpenError {
    Io(std::io::Error),
    OpenFixed(bindb::storage::fixed::OpenError),
    OpenDynamic(bindb::storage::dynamic::OpenError),
    OpenIndexedDynamic(bindb::storage::indexed_dynamic::OpenError),
    OpenBinaryTree(bindb::storage::binary_tree::OpenError),
    OpenSingle(bindb::storage::single::OpenError)
}

impl From<bindb::storage::fixed::OpenError> for OpenError {
    fn from(value: bindb::storage::fixed::OpenError) -> Self {
        Self::OpenFixed(value)
    }
}

impl From<bindb::storage::dynamic::OpenError> for OpenError {
    fn from(value: bindb::storage::dynamic::OpenError) -> Self {
        Self::OpenDynamic(value)
    }
}

impl From<bindb::storage::indexed_dynamic::OpenError> for OpenError {
    fn from(value: bindb::storage::indexed_dynamic::OpenError) -> Self {
        Self::OpenIndexedDynamic(value)
    }
}

impl From<bindb::storage::binary_tree::OpenError> for OpenError {
    fn from(value: bindb::storage::binary_tree::OpenError) -> Self {
        Self::OpenBinaryTree(value)
    }
}

impl From<bindb::storage::single::OpenError> for OpenError {
    fn from(value: bindb::storage::single::OpenError) -> Self {
        Self::OpenSingle(value)
    }
}
// endregion: OpenError

pub struct Value {
    pub users: bindb::storage::IndexedDynamic<user::DbValue>,
    users_name_index: bindb::storage::BinaryTree<ArbNum<4, u64>, Username, ArbNum<4, u64>>,
    meili_client: MeiliClient,
    error_sender: mpsc::Sender<InternalError>
}

const STORAGE_FILES_NAMES: &[&'static str] = &[
    "users_raw_free_locations",
    "users_raw_entries",
    "users_indices",
    "users_free_ids",

    "users_name_index_nodes",
    "users_name_index_free_ids",
    "users_name_index_header",
];

fn open_or_create_file<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
    File::options().read(true).write(true).create(true).open(path)
}

impl Value {
    pub unsafe fn open(
        meili_client: MeiliClient,
        dir_path: impl AsRef<Path>,
        mode: OpenMode,
        error_sender: mpsc::Sender<InternalError>,
    ) -> Result<Self, OpenError> {
        use bindb::storage;
        let dir_path = dir_path.as_ref();
        fn open_file(path: impl AsRef<Path>, mode: OpenMode) -> std::io::Result<File> {
            match mode {
                OpenMode::New => File::options().read(true).write(true).create(true).open(path),
                OpenMode::Existing => File::options().read(true).write(true).open(path)
            }
        };

        macro_rules! open_file {
            ($path: expr) => {
                open_file(dir_path.join($path), mode).map_err(OpenError::Io)?
            };
        }

        Ok(Self {
            meili_client,
            users: storage::IndexedDynamic::open(storage::indexed_dynamic::OpenConfig {
                mode,
                files: storage::indexed_dynamic::OpenFiles {
                    raw_entries: open_file!("users_raw_entries"),
                    raw_free_locations: open_file!("users_raw_free_locations"),
                    indices: open_file!("users_indices"),
                    free_ids: open_file!("users_free_ids")
                },
                max_margins: storage::indexed_dynamic::OpenMaxMargins {
                    raw_entries: 100,
                    raw_free_locations: 20,
                    indices: 20,
                    free_ids: 20,
                }
            })?,
            users_name_index: storage::BinaryTree::open(storage::binary_tree::OpenConfig {
                mode,
                files: storage::binary_tree::OpenFiles {
                    nodes: open_file!("users_name_index_nodes"),
                    free_ids: open_file!("users_name_index_free_ids"),
                    header: open_file!("users_name_index_header"),
                },
                max_margins: storage::binary_tree::OpenMaxMargins {
                    nodes: 20,
                    free_ids: 20,
                },
            })?,
            error_sender,
        })
    }

    async fn send_bindb_error(&self, op: BindbErrorOp, kind: BindbErrorKind) {
        let _ = self.error_sender.send(InternalError::bindb(op, kind)).await;
    }

    async fn send_meili_error(&self, err: meilisearch_sdk::errors::Error) {
        let _ = self.error_sender.send(InternalError::Meili(err)).await;
    }
}

// #[test]
// pub async fn test1() {
//     let meili_client = 
//         meilisearch_sdk::client::Client::new("http://localhost:7700", Some("aSampleMasterKey"))
//         .unwrap();
//     let mut db = unsafe { Value::open(
//         meili_client,
//         &PathBuf::from_str("./test_db").unwrap(), OpenMode::New
//     ) }.unwrap();
//     db.add_user(user::Value {
//         name: "griffpatch".parse().unwrap(),
//         id: 104492,
//         scratch_team: false,
//         status: Cow::Owned("Some status...".to_string()),
//         bio: Cow::Owned("Some cool bio!".to_string()),
//     }).await.unwrap();

//     let griff = db.user_by_name(&"griffpatch".parse().unwrap()).unwrap();
//     println!("{:?}", &griff);
//     println!("{}", griff.name.to_string());
//     assert_eq!(griff.id, 104492);
// }
