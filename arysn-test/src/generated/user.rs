use super::contribution::Contribution;
#[cfg(target_arch = "x86_64")]
use super::role::Role;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub title: Option<String>,
    pub age: i32,
    pub active: bool,
    pub created_at: Option<chrono::DateTime<chrono::Local>>,
    pub roles: Option<Vec<Role>>,
    pub contributions: Option<Vec<Contribution>>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserNew {
    pub id: Option<i64>,
    pub name: String,
    pub title: Option<String>,
    pub age: i32,
    pub active: bool,
    pub created_at: Option<chrono::DateTime<chrono::Local>>,
}
