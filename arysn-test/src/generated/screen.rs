use super::role::Role;
#[derive(Clone, Debug)]
pub struct Screen {
    pub id: i64,
    pub role_id: i64,
    pub name: String,
    pub role: Option<Role>,
}
#[derive(Clone, Debug)]
pub struct ScreenNew {
    pub id: Option<i64>,
    pub role_id: i64,
    pub name: String,
}
