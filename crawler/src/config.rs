
#[derive(Clone, Debug)]
pub struct Run {
    pub db_url: String,
    pub db_auth_key: Option<db::auth::Key>,
    pub initial_user: String,
}