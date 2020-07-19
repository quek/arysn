use super::role::Role;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Screen {
    pub id: i64,
    pub role_id: i64,
    pub name: String,
    pub role: Option<Role>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScreenNew {
    pub id: Option<i64>,
    pub role_id: i64,
    pub name: String,
}
