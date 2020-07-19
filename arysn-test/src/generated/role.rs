use super::screen::Screen;
use super::user::User;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, ToSql, FromSql, Deserialize, Serialize)]
#[postgres(name = "role_type")]
pub enum RoleType {
    #[postgres(name = "admin")]
    Admin,
    #[postgres(name = "user")]
    User,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Role {
    pub id: i64,
    pub user_id: i64,
    pub role_type: RoleType,
    pub screens: Option<Vec<Screen>>,
    pub user: Option<User>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoleNew {
    pub id: Option<i64>,
    pub user_id: i64,
    pub role_type: RoleType,
}
