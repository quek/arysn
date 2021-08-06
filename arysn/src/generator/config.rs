use std::path::Path;

#[derive(Debug)]
pub struct Config {
    pub path: &'static str,
    pub table_name: &'static str,
    pub struct_name: &'static str,
    pub has_many: Vec<HasManyConfig>,
    pub has_one: Vec<HasOneConfig>,
    pub belongs_to: Vec<BelongsToConfig>,
}

impl Config {
    pub fn mod_name(&self) -> String {
        Path::new(self.path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
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
