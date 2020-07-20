use super::screen::Screen;
use super::user::User;
#[cfg(target_arch = "x86_64")]
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(
    target_arch = "x86_64",
    derive(FromSql, ToSql),
    postgres(name = "role_type")
)]
pub enum RoleType {
    #[cfg_attr(target_arch = "x86_64", postgres(name = "admin"))]
    Admin,
    #[cfg_attr(target_arch = "x86_64", postgres(name = "user"))]
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
