use anyhow::Result;
use arysn::generator::config::{BelongsToConfig, Config, HasManyConfig};
use arysn::generator::define_ar;
use quote::format_ident;
use std::process::Command;

fn main() -> Result<()> {
    let config = Config {
        path: "src/generated/user.rs",
        table_name: "users",
        struct_name: format_ident!("{}", "User"),
        has_many: Some(HasManyConfig {
            field: format_ident!("{}", "roles"),
            struct_name: format_ident!("{}", "Role"),
        }),
        belongs_to: None,
    };
    define_ar(&config)?;
    Command::new("rustfmt")
        .arg("--edition")
        .arg("2018")
        .arg(config.path)
        .output()?;

    let config = Config {
        path: "src/generated/role.rs",
        table_name: "roles",
        struct_name: format_ident!("{}", "Role"),
        has_many: Some(HasManyConfig {
            field: format_ident!("{}", "screens"),
            struct_name: format_ident!("{}", "Screen"),
        }),
        belongs_to: Some(BelongsToConfig {
            field: format_ident!("{}", "user"),
            struct_name: format_ident!("{}", "User"),
        }),
    };
    define_ar(&config)?;
    Command::new("rustfmt")
        .arg("--edition")
        .arg("2018")
        .arg(config.path)
        .output()?;

    let config = Config {
        path: "src/generated/screen.rs",
        table_name: "screens",
        struct_name: format_ident!("{}", "Screen"),
        has_many: None,
        belongs_to: Some(BelongsToConfig {
            field: format_ident!("{}", "role"),
            struct_name: format_ident!("{}", "Role"),
        }),
    };
    define_ar(&config)?;
    Command::new("rustfmt")
        .arg("--edition")
        .arg("2018")
        .arg(config.path)
        .output()?;

    Ok(())
}
