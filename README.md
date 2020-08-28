arysn is Rust ORM code generator.

see arysn-test/build.rs, arysn-test/tests

# Connection

To access Postgresql, [tokio-postgres](https://crates.io/crates/tokio-postgres) to be used.
You can also use [deadpool-postgres](https://crates.io/crates/deadpool-postgres).

# Code generation

`define_ar` is used to generate the code.

``` rust
use arysn::generator::config::{BelongsToConfig, Config, HasManyConfig};
use arysn::generator::define_ar;

fn main() {
    define_ar(&Config {
        path: "src/generated/user.rs",
        table_name: "users",
        struct_name: "User",
        has_many: vec![
            HasManyConfig {
                field: "roles",
                struct_name: "Role",
                foreign_key: "user_id",
            },
        ],
        has_one: vec![],
        belongs_to: vec![],
    }).unwrap();

    define_ar(&Config {
        path: "src/generated/role.rs",
        table_name: "roles",
        struct_name: "Role",
        has_many: vec![],
        has_one: vec![],
        belongs_to: vec![BelongsToConfig {
            field: "user",
            struct_name: "User",
            foreign_key: "user_id",
        }],
    }).unwrap();
```

The following four files will be generated.

- user.rs
- user_impl.rs
- role.rs
- role_impl.rs

user.rs and role.rs are intended to be used in frontend applications such as Yew.
user_impl.rs and role_impl.rs contain DB access code.

# Query

``` rust
let conn = ... // tokio-postgres or deadpool-postgres client
let users: Vec<User> = User::select().active().eq(true).load(&conn).await?;

let user: User = User::select().id(1).first(&conn).await?;

use arysn::Optional;
let user: Option<Vec> = User::select().id(1).first(&conn).await.optional()?;
```

## Join

``` rust
let users: Vec<User> = User::select().roles(|role| role.role_type().eq(RoleType::Admin))
    .load(&conn).await?;
```

## N+1

``` rust
let users: Vec<User> = User::select().roles(|role| role.preload())
    .load(&conn).await?;
```

SQL looks like this

``` sql
SELECT * FORM users;
SELECT * FROM rolse WERE WHERE id IN (....);
```

The conditions before the preload are used to join.

``` rust
let users: Vec<User> = User::select()
    .roles(|role| role.role_type().eq(RoleType::Admin).preload())
    .load(&conn).await?;
```

SQL looks like this

``` sql
SELECT * FORM users INNER JOIN roles ON roles.user_id=users.id
    WHERE roles.role_types='admin';
SELECT * FROM rolse WERE WHERE role_types='admin' id IN (....);
```

Conditions after the preload are not used when joining.

``` rust
let users: Vec<User> = User::select()
    .roles(|role| role.preload().role_type().eq(RoleType::Admin))
    .load(&conn).await?;
```

SQL looks like this

``` sql
SELECT * FORM users;
SELECT * FROM rolse WERE WHERE role_types='admin' id IN (....);
```

# UUID

To use the UUID, you need to specify features for tokio-postgres and uuid.

Cargo.toml

``` toml
[dependencies]
tokio-postgres = { version = "0.5", features = ["with-chrono-0_4", "with-uuid-0_8"] }
uuid = { version = "0.8", features = ["serde"] }
```

# PostGIS

Only PostGIS `POINT` is supported.
You must specify `gis` featuers and `postgis` crate is required.

Cargo.toml

``` toml
[dependencies]
arysn = { version = "0.1.8", features = ["gis"] }
postgis = "0.7"
```
