use super::contribution::Contribution;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub contributions: Option<Vec<Contribution>>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectNew {
    pub id: Option<i64>,
    pub name: String,
}
