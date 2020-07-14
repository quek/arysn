use super::user::{User, UserBuilder};
use arysn::prelude::*;
#[derive(Clone, Debug)]
pub struct Role {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
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
            user: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct RoleBuilder {
    pub from: String,
    pub filters: Vec<Filter>,
    pub preload: bool,
    pub user_bulider: Option<Box<UserBuilder>>,
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
    pub fn user<F>(&self, f: F) -> RoleBuilder
    where
        F: FnOnce(&UserBuilder) -> UserBuilder,
    {
        RoleBuilder {
            user_bulider: Some(Box::new(f(self
                .user_bulider
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
    pub async fn first(&self, client: &tokio_postgres::Client) -> anyhow::Result<Role> {
        let params = self.select_params();
        let row = client
            .query_one(self.select_sql().as_str(), &params[..])
            .await?;
        let x: Role = Role::from(row);
        Ok(x)
    }
    pub async fn load(&self, client: &tokio_postgres::Client) -> anyhow::Result<Vec<Role>> {
        let params = self.select_params();
        let rows = client
            .query(self.select_sql().as_str(), &params[..])
            .await?;
        let mut xs: Vec<Role> = rows.into_iter().map(|row| Role::from(row)).collect();
        Ok(xs)
    }
}
impl BuilderTrait for RoleBuilder {
    fn select(&self) -> String {
        "roles".to_string()
    }
    fn from(&self) -> String {
        let mut result = self.from.clone();
        if self.user_bulider.is_some() {
            result.push_str(" INNER JOIN users ON users.id = roles.user_id");
        }
        result
    }
    fn filters(&self) -> Vec<&Filter> {
        let mut result: Vec<&Filter> = self.filters.iter().collect();
        result.append(
            &mut self
                .user_bulider
                .as_ref()
                .map_or(vec![], |x| x.filters.iter().collect::<Vec<&Filter>>()),
        );
        result
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
