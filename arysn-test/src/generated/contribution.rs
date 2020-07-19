use super::project::Project;
use super::user::User;
#[derive(Clone, Debug)]
pub struct Contribution {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub project: Option<Project>,
    pub user: Option<User>,
}
#[derive(Clone, Debug)]
pub struct ContributionNew {
    pub id: Option<i64>,
    pub project_id: i64,
    pub user_id: i64,
}
