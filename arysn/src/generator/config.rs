#[derive(Debug)]
pub struct Config {
    pub path: &'static str,
    pub table_name: &'static str,
    pub struct_name: &'static str,
    pub has_many: Vec<HasManyConfig>,
    pub has_one: Vec<HasOneConfig>,
    pub belongs_to: Vec<BelongsToConfig>,
}

#[derive(Debug)]
pub struct HasManyConfig {
    pub field: &'static str,
    pub struct_name: &'static str,
    pub foreign_key: &'static str,
}

#[derive(Debug)]
pub struct HasOneConfig {
    pub field: &'static str,
    pub struct_name: &'static str,
    pub foreign_key: &'static str,
}

#[derive(Debug)]
pub struct BelongsToConfig {
    pub field: &'static str,
    pub struct_name: &'static str,
    pub foreign_key: &'static str,
}
