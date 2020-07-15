use anyhow::Result;
use arysn::generator::config::{BelongsToConfig, Config, HasManyConfig};
use arysn::generator::define_ar;
use quote::format_ident;

fn main() -> Result<()> {
    let config = Config {
        path: "src/generated/user.rs",
        table_name: "users",
        struct_name: format_ident!("{}", "User"),
        has_many: vec![
            HasManyConfig {
                field: format_ident!("{}", "roles"),
                struct_name: format_ident!("{}", "Role"),
            },
            HasManyConfig {
                field: format_ident!("{}", "contributions"),
                struct_name: format_ident!("{}", "Contribution"),
            },
        ],
        belongs_to: vec![],
    };
    define_ar(&config)?;

    let config = Config {
        path: "src/generated/role.rs",
        table_name: "roles",
        struct_name: format_ident!("{}", "Role"),
        has_many: vec![HasManyConfig {
            field: format_ident!("{}", "screens"),
            struct_name: format_ident!("{}", "Screen"),
        }],
        belongs_to: vec![BelongsToConfig {
            field: format_ident!("{}", "user"),
            struct_name: format_ident!("{}", "User"),
        }],
    };
    define_ar(&config)?;

    let config = Config {
        path: "src/generated/screen.rs",
        table_name: "screens",
        struct_name: format_ident!("{}", "Screen"),
        has_many: vec![],
        belongs_to: vec![BelongsToConfig {
            field: format_ident!("{}", "role"),
            struct_name: format_ident!("{}", "Role"),
        }],
    };
    define_ar(&config)?;

    define_ar(&Config {
        path: "src/generated/project.rs",
        table_name: "projects",
        struct_name: format_ident!("{}", "Project"),
        has_many: vec![HasManyConfig {
            field: format_ident!("{}", "contributions"),
            struct_name: format_ident!("{}", "Contribution"),
        }],
        belongs_to: vec![],
    });

    define_ar(&Config {
        path: "src/generated/contribution.rs",
        table_name: "contributions",
        struct_name: format_ident!("{}", "Contribution"),
        has_many: vec![],
        belongs_to: vec![
            BelongsToConfig {
                field: format_ident!("{}", "project"),
                struct_name: format_ident!("{}", "Project"),
            },
            BelongsToConfig {
                field: format_ident!("{}", "user"),
                struct_name: format_ident!("{}", "User"),
            },
        ],
    });

    Ok(())
}
