use super::contribution::Contribution;
#[derive(Clone, Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub contributions: Option<Vec<Contribution>>,
}
#[derive(Clone, Debug)]
pub struct ProjectNew {
    pub id: Option<i64>,
    pub name: String,
}
