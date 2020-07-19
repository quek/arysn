use super::screen::Screen;
use super::user::User;
use postgres_types::{FromSql, ToSql};
#[derive(Debug, Clone, ToSql, FromSql)]
#[postgres(name = "role_type")]
pub enum RoleType {
    #[postgres(name = "admin")]
    Admin,
    #[postgres(name = "user")]
    User,
}
#[derive(Clone, Debug)]
pub struct Role {
    pub id: i64,
    pub user_id: i64,
    pub role_type: RoleType,
    pub screens: Option<Vec<Screen>>,
    pub user: Option<User>,
}
#[derive(Clone, Debug)]
pub struct RoleNew {
    pub id: Option<i64>,
    pub user_id: i64,
    pub role_type: RoleType,
}
