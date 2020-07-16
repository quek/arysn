use super::screen::{Screen, ScreenBuilder};
use super::user::{User, UserBuilder};
use arysn::prelude::*;
use async_recursion::async_recursion;
#[derive(Clone, Debug)]
pub struct Role {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub screens: Option<Vec<Screen>>,
    pub user: Option<User>,
}
#[derive(Clone, Debug)]
pub struct RoleNew {
    pub id: Option<i64>,
    pub user_id: i64,
    pub name: String,
}
impl Role {
    pub fn select() -> RoleBuilder {
        RoleBuilder {
            from: "roles".to_string(),
            ..RoleBuilder::default()
        }
    }
    pub async fn delete(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .execute("DELETE FROM roles WHERE id = $1", &[&self.id])
            .await?;
        Ok(())
    }
    pub async fn update(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .execute(
                "UPDATE roles SET user_id = $1, name = $2 WHERE id = $3",
                &[&self.user_id, &self.name, &self.id],
            )
            .await?;
        Ok(())
    }
}
impl RoleNew {
    pub async fn insert(&self, client: &tokio_postgres::Client) -> anyhow::Result<Role> {
        let mut target_columns: Vec<&str> = vec![];
        target_columns.push(stringify!(user_id));
        target_columns.push(stringify!(name));
        let target_columns = target_columns.join(", ");
        let mut bind_count: i32 = 0;
        bind_count += 1;
        bind_count += 1;
        let binds = (1..=bind_count)
            .map(|i| format!("${}", i))
            .collect::<Vec<_>>()
            .join(", ");
        let statement = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            "roles", target_columns, binds
        );
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        params.push(&self.user_id);
        params.push(&self.name);
        let row = client.query_one(statement.as_str(), &params[..]).await?;
        Ok(row.into())
    }
}
impl From<tokio_postgres::row::Row> for Role {
    fn from(row: tokio_postgres::row::Row) -> Self {
        Self {
            id: row.get(0usize),
            user_id: row.get(1usize),
            name: row.get(2usize),
            screens: None,
            user: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct RoleBuilder {
    pub from: String,
    pub filters: Vec<Filter>,
    pub preload: bool,
    pub order: String,
    pub screens_builder: Option<Box<ScreenBuilder>>,
    pub user_builder: Option<Box<UserBuilder>>,
}
impl RoleBuilder {
    pub fn id(&self) -> RoleBuilder_id {
        RoleBuilder_id {
            builder: self.clone(),
        }
    }
    pub fn user_id(&self) -> RoleBuilder_user_id {
        RoleBuilder_user_id {
            builder: self.clone(),
        }
    }
    pub fn name(&self) -> RoleBuilder_name {
        RoleBuilder_name {
            builder: self.clone(),
        }
    }
    pub fn screens<F>(&self, f: F) -> RoleBuilder
    where
        F: FnOnce(&ScreenBuilder) -> ScreenBuilder,
    {
        RoleBuilder {
            screens_builder: Some(Box::new(f(self
                .screens_builder
                .as_ref()
                .unwrap_or(&Default::default())))),
            ..self.clone()
        }
    }
    pub fn user<F>(&self, f: F) -> RoleBuilder
    where
        F: FnOnce(&UserBuilder) -> UserBuilder,
    {
        RoleBuilder {
            user_builder: Some(Box::new(f(self
                .user_builder
                .as_ref()
                .unwrap_or(&Default::default())))),
            ..self.clone()
        }
    }
    pub fn order<T: AsRef<str>>(&self, value: T) -> Self {
        Self {
            order: value.as_ref().to_string(),
            ..self.clone()
        }
    }
    pub fn preload(&self) -> Self {
        Self {
            preload: true,
            ..self.clone()
        }
    }
    pub async fn first(&self, client: &tokio_postgres::Client) -> anyhow::Result<Role> {
        let params = self.select_params();
        let row = client
            .query_one(self.select_sql().as_str(), &params[..])
            .await?;
        let x: Role = Role::from(row);
        Ok(x)
    }
    #[async_recursion]
    pub async fn load(&self, client: &tokio_postgres::Client) -> anyhow::Result<Vec<Role>> {
        let params = self.select_params();
        let rows = client
            .query(self.select_sql().as_str(), &params[..])
            .await?;
        let mut result: Vec<Role> = rows.into_iter().map(|row| Role::from(row)).collect();
        if let Some(builder) = &self.screens_builder {
            if builder.preload {
                let ids = result.iter().map(|x| x.id).collect::<Vec<_>>();
                let children_builder = Screen::select().role_id().eq_any(ids);
                let children_builder = ScreenBuilder {
                    from: children_builder.from,
                    filters: children_builder.filters,
                    ..(**builder).clone()
                };
                let children = children_builder.load(client).await?;
                result.iter_mut().for_each(|x| {
                    let mut ys = vec![];
                    for child in children.iter() {
                        if x.id == child.role_id {
                            ys.push(child.clone());
                        }
                    }
                    x.screens = Some(ys);
                });
            }
        }
        if let Some(builder) = &self.user_builder {
            if builder.preload {
                let ids = result.iter().map(|x| x.user_id).collect::<Vec<_>>();
                let parents_builder = User::select().id().eq_any(ids);
                let parents_builder = UserBuilder {
                    from: parents_builder.from,
                    filters: parents_builder.filters,
                    ..(**builder).clone()
                };
                let parents = parents_builder.load(client).await?;
                result.iter_mut().for_each(|x| {
                    for parent in parents.iter() {
                        if x.user_id == parent.id {
                            x.user = Some(parent.clone());
                            break;
                        }
                    }
                });
            }
        }
        Ok(result)
    }
}
impl BuilderTrait for RoleBuilder {
    fn select(&self) -> String {
        "roles".to_string()
    }
    fn from(&self) -> String {
        let mut result: Vec<String> = vec![self.from.clone()];
        self.join(&mut result);
        result.join(" ")
    }
    fn join(&self, join_parts: &mut Vec<String>) {
        if let Some(builder) = &self.screens_builder {
            join_parts.push("INNER JOIN screens ON screens.role_id = roles.id".to_string());
            builder.join(join_parts);
        }
        if let Some(builder) = &self.user_builder {
            join_parts.push("INNER JOIN users ON users.id = roles.user_id".to_string());
            builder.join(join_parts);
        }
    }
    fn filters(&self) -> Vec<&Filter> {
        let mut result: Vec<&Filter> = self.filters.iter().collect();
        if let Some(builder) = &self.screens_builder {
            result.append(&mut builder.filters());
        }
        if let Some(builder) = &self.user_builder {
            result.append(&mut builder.filters());
        }
        result
    }
    fn order_part(&self) -> String {
        self.order.clone()
    }
}
#[allow(non_camel_case_types)]
pub struct RoleBuilder_id {
    pub builder: RoleBuilder,
}
impl RoleBuilder_id {
    pub fn eq(&self, value: i64) -> RoleBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "roles".to_string(),
            name: stringify!(id).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        RoleBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i64>) -> RoleBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "roles".to_string(),
            name: stringify!(id).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        RoleBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct RoleBuilder_user_id {
    pub builder: RoleBuilder,
}
impl RoleBuilder_user_id {
    pub fn eq(&self, value: i64) -> RoleBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "roles".to_string(),
            name: stringify!(user_id).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        RoleBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i64>) -> RoleBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "roles".to_string(),
            name: stringify!(user_id).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        RoleBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct RoleBuilder_name {
    pub builder: RoleBuilder,
}
impl RoleBuilder_name {
    pub fn eq(&self, value: String) -> RoleBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "roles".to_string(),
            name: stringify!(name).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        RoleBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<String>) -> RoleBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "roles".to_string(),
            name: stringify!(name).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        RoleBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
