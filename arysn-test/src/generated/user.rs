use super::role::{Role, RoleBuilder};
use arysn::prelude::*;
use async_recursion::async_recursion;
#[derive(Clone, Debug)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub title: Option<String>,
    pub age: i32,
    pub active: bool,
    pub created_at: Option<chrono::DateTime<chrono::Local>>,
    pub roles: Option<Vec<Role>>,
}
#[derive(Clone, Debug)]
pub struct UserNew {
    pub id: Option<i64>,
    pub name: String,
    pub title: Option<String>,
    pub age: i32,
    pub active: bool,
    pub created_at: Option<chrono::DateTime<chrono::Local>>,
}
impl User {
    pub fn select() -> UserBuilder {
        UserBuilder {
            from: "users".to_string(),
            ..UserBuilder::default()
        }
    }
    pub async fn delete(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .execute("DELETE FROM users WHERE id = $1", &[&self.id])
            .await?;
        Ok(())
    }
    pub async fn update(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client . execute ( "UPDATE users SET name = $1, title = $2, age = $3, active = $4, created_at = $5 WHERE id = $6" , & [ & self . name , & self . title , & self . age , & self . active , & self . created_at , & self . id ] ) . await ? ;
        Ok(())
    }
}
impl UserNew {
    pub async fn insert(&self, client: &tokio_postgres::Client) -> anyhow::Result<User> {
        let mut target_columns: Vec<&str> = vec![];
        target_columns.push(stringify!(name));
        target_columns.push(stringify!(title));
        target_columns.push(stringify!(age));
        target_columns.push(stringify!(active));
        if self.created_at.is_some() {
            target_columns.push(stringify!(created_at));
        }
        let target_columns = target_columns.join(", ");
        let mut bind_count: i32 = 0;
        bind_count += 1;
        bind_count += 1;
        bind_count += 1;
        bind_count += 1;
        if self.created_at.is_some() {
            bind_count += 1;
        }
        let binds = (1..=bind_count)
            .map(|i| format!("${}", i))
            .collect::<Vec<_>>()
            .join(", ");
        let statement = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            "users", target_columns, binds
        );
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        params.push(&self.name);
        params.push(&self.title);
        params.push(&self.age);
        params.push(&self.active);
        if self.created_at.is_some() {
            params.push(&self.created_at);
        }
        let row = client.query_one(statement.as_str(), &params[..]).await?;
        Ok(row.into())
    }
}
impl From<tokio_postgres::row::Row> for User {
    fn from(row: tokio_postgres::row::Row) -> Self {
        Self {
            id: row.get(0usize),
            name: row.get(1usize),
            title: row.get(2usize),
            age: row.get(3usize),
            active: row.get(4usize),
            created_at: row.get(5usize),
            roles: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct UserBuilder {
    pub from: String,
    pub filters: Vec<Filter>,
    pub preload: bool,
    pub roles_builder: Option<Box<RoleBuilder>>,
}
impl UserBuilder {
    pub fn id(&self) -> UserBuilder_id {
        UserBuilder_id {
            builder: self.clone(),
        }
    }
    pub fn name(&self) -> UserBuilder_name {
        UserBuilder_name {
            builder: self.clone(),
        }
    }
    pub fn title(&self) -> UserBuilder_title {
        UserBuilder_title {
            builder: self.clone(),
        }
    }
    pub fn age(&self) -> UserBuilder_age {
        UserBuilder_age {
            builder: self.clone(),
        }
    }
    pub fn active(&self) -> UserBuilder_active {
        UserBuilder_active {
            builder: self.clone(),
        }
    }
    pub fn created_at(&self) -> UserBuilder_created_at {
        UserBuilder_created_at {
            builder: self.clone(),
        }
    }
    pub fn roles<F>(&self, f: F) -> UserBuilder
    where
        F: FnOnce(&RoleBuilder) -> RoleBuilder,
    {
        UserBuilder {
            roles_builder: Some(Box::new(f(self
                .roles_builder
                .as_ref()
                .unwrap_or(&Default::default())))),
            ..self.clone()
        }
    }
    pub fn preload(&self) -> Self {
        Self {
            preload: true,
            ..self.clone()
        }
    }
    pub async fn first(&self, client: &tokio_postgres::Client) -> anyhow::Result<User> {
        let params = self.select_params();
        let row = client
            .query_one(self.select_sql().as_str(), &params[..])
            .await?;
        let x: User = User::from(row);
        Ok(x)
    }
    #[async_recursion]
    pub async fn load(&self, client: &tokio_postgres::Client) -> anyhow::Result<Vec<User>> {
        let params = self.select_params();
        let rows = client
            .query(self.select_sql().as_str(), &params[..])
            .await?;
        let mut xs: Vec<User> = rows.into_iter().map(|row| User::from(row)).collect();
        if self.roles_builder.as_ref().map_or(false, |x| x.preload) {
            let ids = xs.iter().map(|x| x.id).collect::<Vec<_>>();
            let zs = Role::select().user_id().eq_any(ids).load(client).await?;
            xs.iter_mut().for_each(|x| {
                let mut ys = vec![];
                for z in zs.iter() {
                    if x.id == z.user_id {
                        ys.push(z.clone());
                    }
                }
                x.roles = Some(ys);
            });
        }
        Ok(xs)
    }
}
impl BuilderTrait for UserBuilder {
    fn select(&self) -> String {
        "users".to_string()
    }
    fn from(&self) -> String {
        let mut result = self.from.clone();
        if self.roles_builder.is_some() {
            result.push_str(" INNER JOIN roles ON roles.user_id = users.id");
        }
        result
    }
    fn filters(&self) -> Vec<&Filter> {
        let mut result: Vec<&Filter> = self.filters.iter().collect();
        result.append(
            &mut self
                .roles_builder
                .as_ref()
                .map_or(vec![], |x| x.filters.iter().collect::<Vec<&Filter>>()),
        );
        result
    }
}
#[allow(non_camel_case_types)]
pub struct UserBuilder_id {
    pub builder: UserBuilder,
}
impl UserBuilder_id {
    pub fn eq(&self, value: i64) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(id).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i64>) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(id).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct UserBuilder_name {
    pub builder: UserBuilder,
}
impl UserBuilder_name {
    pub fn eq(&self, value: String) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(name).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<String>) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(name).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct UserBuilder_title {
    pub builder: UserBuilder,
}
impl UserBuilder_title {
    pub fn eq(&self, value: String) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(title).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<String>) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(title).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct UserBuilder_age {
    pub builder: UserBuilder,
}
impl UserBuilder_age {
    pub fn eq(&self, value: i32) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(age).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i32>) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(age).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct UserBuilder_active {
    pub builder: UserBuilder,
}
impl UserBuilder_active {
    pub fn eq(&self, value: bool) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(active).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<bool>) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(active).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct UserBuilder_created_at {
    pub builder: UserBuilder,
}
impl UserBuilder_created_at {
    pub fn eq(&self, value: chrono::DateTime<chrono::Local>) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(created_at).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<chrono::DateTime<chrono::Local>>) -> UserBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "users".to_string(),
            name: stringify!(created_at).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        UserBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
