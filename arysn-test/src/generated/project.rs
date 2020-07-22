use super::contribution::Contribution;
use super::user::User;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub parent_project_id: Option<i64>,
    pub create_user_id: i64,
    pub update_user_id: i64,
    pub contributions: Option<Vec<Contribution>>,
    pub create_user: Option<User>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectNew {
    pub id: Option<i64>,
    pub name: String,
    pub parent_project_id: Option<i64>,
    pub create_user_id: i64,
    pub update_user_id: i64,
}
