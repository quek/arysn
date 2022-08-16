use anyhow::Result;
use arysn::generator::config::{BelongsToConfig, Config, HasManyConfig, HasOneConfig};
use arysn::generator::define_ar;

fn main() -> Result<()> {
    define_ar(
        "src/generated".into(),
        vec![
            "Clone".to_string(),
            "Debug".to_string(),
            "PartialEq".to_string(),
            "Deserialize".to_string(),
            "Serialize".to_string(),
        ],
        vec![
            Config {
                path: "simple.rs",
                table_name: "simples",
                struct_name: "Simple",
                has_many: vec![],
                has_one: vec![],
                belongs_to: vec![],
            },
            Config {
                path: "user.rs",
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
                    HasManyConfig {
                        field: "create_projects",
                        struct_name: "Project",
                        foreign_key: "create_user_id",
                    },
                    HasManyConfig {
                        field: "update_projects",
                        struct_name: "Project",
                        foreign_key: "update_user_id",
                    },
                ],
                has_one: vec![HasOneConfig {
                    field: "profile",
                    struct_name: "Profile",
                    foreign_key: "user_id",
                }],
                belongs_to: vec![],
            },
            Config {
                path: "profile.rs",
                table_name: "profiles",
                struct_name: "Profile",
                has_many: vec![],
                has_one: vec![],
                belongs_to: vec![BelongsToConfig {
                    field: "user",
                    struct_name: "User",
                    foreign_key: "user_id",
                }],
            },
            Config {
                path: "role.rs",
                table_name: "roles",
                struct_name: "Role",
                has_many: vec![HasManyConfig {
                    field: "screens",
                    struct_name: "Screen",
                    foreign_key: "role_id",
                }],
                has_one: vec![],
                belongs_to: vec![BelongsToConfig {
                    field: "user",
                    struct_name: "User",
                    foreign_key: "user_id",
                }],
            },
            Config {
                path: "screen.rs",
                table_name: "screens",
                struct_name: "Screen",
                has_many: vec![],
                has_one: vec![],
                belongs_to: vec![BelongsToConfig {
                    field: "role",
                    struct_name: "Role",
                    foreign_key: "role_id",
                }],
            },
            Config {
                path: "project.rs",
                table_name: "projects",
                struct_name: "Project",
                has_many: vec![
                    HasManyConfig {
                        field: "contributions",
                        struct_name: "Contribution",
                        foreign_key: "project_id",
                    },
                    HasManyConfig {
                        field: "child_projects",
                        struct_name: "Project",
                        foreign_key: "parent_project_id",
                    },
                ],
                has_one: vec![],
                belongs_to: vec![
                    BelongsToConfig {
                        field: "create_user",
                        struct_name: "User",
                        foreign_key: "create_user_id",
                    },
                    BelongsToConfig {
                        field: "update_user",
                        struct_name: "User",
                        foreign_key: "update_user_id",
                    },
                    BelongsToConfig {
                        field: "check_user",
                        struct_name: "User",
                        foreign_key: "check_user_id",
                    },
                ],
            },
            Config {
                path: "contribution.rs",
                table_name: "contributions",
                struct_name: "Contribution",
                has_many: vec![],
                has_one: vec![],
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
            },
            #[cfg(any(feature = "gis"))]
            Config {
                path: "gis_thing.rs",
                table_name: "gis_things",
                struct_name: "GisThing",
                has_many: vec![],
                has_one: vec![],
                belongs_to: vec![],
            },
        ],
    )?;

    Ok(())
}
