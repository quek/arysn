use proc_macro2::Ident;

#[derive(Debug)]
pub struct Config {
    pub path: &'static str,
    pub table_name: &'static str,
    pub struct_name: Ident,
    pub has_many: Option<HasManyConfig>,
    pub belongs_to: Option<BelongsToConfig>,
}

#[derive(Debug)]
pub struct HasManyConfig {
    pub field: Ident,
    pub struct_name: Ident,
}

#[derive(Debug)]
pub struct BelongsToConfig {
    pub field: Ident,
    pub struct_name: Ident,
}
