use anyhow::Result;
use arysn::generator::config::{BelongsToConfig, Config, HasManyConfig};
use arysn::generator::define_ar;

fn main() -> Result<()> {
    let config = Config {
        path: "src/generated/user.rs",
        table_name: "users",
        struct_name: "User",
        has_many: vec![
            HasManyConfig {
                field: "roles",
                struct_name: "Role",
                foreign_key: "user_id",
            },
            HasManyConfig {
                field: "contributions",
                struct_name: "Contribution",
                foreign_key: "user_id",
            },
        ],
        belongs_to: vec![],
    };
    define_ar(&config)?;

    let config = Config {
        path: "src/generated/role.rs",
        table_name: "roles",
        struct_name: "Role",
        has_many: vec![HasManyConfig {
            field: "screens",
            struct_name: "Screen",
            foreign_key: "role_id",
        }],
        belongs_to: vec![BelongsToConfig {
            field: "user",
            struct_name: "User",
            foreign_key: "user_id",
        }],
    };
    define_ar(&config)?;

    let config = Config {
        path: "src/generated/screen.rs",
        table_name: "screens",
        struct_name: "Screen",
        has_many: vec![],
        belongs_to: vec![BelongsToConfig {
            field: "role",
            struct_name: "Role",
            foreign_key: "role_id",
        }],
    };
    define_ar(&config)?;

    define_ar(&Config {
        path: "src/generated/project.rs",
        table_name: "projects",
        struct_name: "Project",
        has_many: vec![HasManyConfig {
            field: "contributions",
            struct_name: "Contribution",
            foreign_key: "project_id",
        }],
        belongs_to: vec![BelongsToConfig {
            field: "create_user",
            struct_name: "User",
            foreign_key: "create_user_id",
        }],
    })?;

    define_ar(&Config {
        path: "src/generated/contribution.rs",
        table_name: "contributions",
        struct_name: "Contribution",
        has_many: vec![],
        belongs_to: vec![
            BelongsToConfig {
                field: "project",
                struct_name: "Project",
                foreign_key: "project_id",
            },
            BelongsToConfig {
                field: "user",
                struct_name: "User",
                foreign_key: "user_id",
            },
        ],
    })?;

    Ok(())
}
