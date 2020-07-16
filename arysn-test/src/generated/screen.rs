use super::role::{Role, RoleBuilder};
use arysn::prelude::*;
use async_recursion::async_recursion;
#[derive(Clone, Debug)]
pub struct Screen {
    pub id: i64,
    pub role_id: i64,
    pub name: String,
    pub role: Option<Role>,
}
#[derive(Clone, Debug)]
pub struct ScreenNew {
    pub id: Option<i64>,
    pub role_id: i64,
    pub name: String,
}
impl Screen {
    pub fn select() -> ScreenBuilder {
        ScreenBuilder {
            from: "screens".to_string(),
            ..ScreenBuilder::default()
        }
    }
    pub async fn delete(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .execute("DELETE FROM screens WHERE id = $1", &[&self.id])
            .await?;
        Ok(())
    }
    pub async fn update(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .execute(
                "UPDATE screens SET role_id = $1, name = $2 WHERE id = $3",
                &[&self.role_id, &self.name, &self.id],
            )
            .await?;
        Ok(())
    }
}
impl ScreenNew {
    pub async fn insert(&self, client: &tokio_postgres::Client) -> anyhow::Result<Screen> {
        let mut target_columns: Vec<&str> = vec![];
        target_columns.push(stringify!(role_id));
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
            "screens", target_columns, binds
        );
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        params.push(&self.role_id);
        params.push(&self.name);
        let row = client.query_one(statement.as_str(), &params[..]).await?;
        Ok(row.into())
    }
}
impl From<tokio_postgres::row::Row> for Screen {
    fn from(row: tokio_postgres::row::Row) -> Self {
        Self {
            id: row.get(0usize),
            role_id: row.get(1usize),
            name: row.get(2usize),
            role: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct ScreenBuilder {
    pub from: String,
    pub filters: Vec<Filter>,
    pub preload: bool,
    pub order: String,
    pub role_builder: Option<Box<RoleBuilder>>,
}
impl ScreenBuilder {
    pub fn id(&self) -> ScreenBuilder_id {
        ScreenBuilder_id {
            builder: self.clone(),
        }
    }
    pub fn role_id(&self) -> ScreenBuilder_role_id {
        ScreenBuilder_role_id {
            builder: self.clone(),
        }
    }
    pub fn name(&self) -> ScreenBuilder_name {
        ScreenBuilder_name {
            builder: self.clone(),
        }
    }
    pub fn role<F>(&self, f: F) -> ScreenBuilder
    where
        F: FnOnce(&RoleBuilder) -> RoleBuilder,
    {
        ScreenBuilder {
            role_builder: Some(Box::new(f(self
                .role_builder
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
    pub async fn first(&self, client: &tokio_postgres::Client) -> anyhow::Result<Screen> {
        let params = self.select_params();
        let row = client
            .query_one(self.select_sql().as_str(), &params[..])
            .await?;
        let x: Screen = Screen::from(row);
        Ok(x)
    }
    #[async_recursion]
    pub async fn load(&self, client: &tokio_postgres::Client) -> anyhow::Result<Vec<Screen>> {
        let params = self.select_params();
        let rows = client
            .query(self.select_sql().as_str(), &params[..])
            .await?;
        let mut result: Vec<Screen> = rows.into_iter().map(|row| Screen::from(row)).collect();
        if let Some(builder) = &self.role_builder {
            if builder.preload {
                let ids = result.iter().map(|x| x.role_id).collect::<Vec<_>>();
                let parents_builder = Role::select().id().eq_any(ids);
                let parents_builder = RoleBuilder {
                    from: parents_builder.from,
                    filters: parents_builder.filters,
                    ..(**builder).clone()
                };
                let parents = parents_builder.load(client).await?;
                result.iter_mut().for_each(|x| {
                    for parent in parents.iter() {
                        if x.role_id == parent.id {
                            x.role = Some(parent.clone());
                            break;
                        }
                    }
                });
            }
        }
        Ok(result)
    }
}
impl BuilderTrait for ScreenBuilder {
    fn select(&self) -> String {
        "screens".to_string()
    }
    fn from(&self) -> String {
        let mut result: Vec<String> = vec![self.from.clone()];
        self.join(&mut result);
        result.join(" ")
    }
    fn join(&self, join_parts: &mut Vec<String>) {
        if let Some(builder) = &self.role_builder {
            join_parts.push("INNER JOIN roles ON roles.id = screens.role_id".to_string());
            builder.join(join_parts);
        }
    }
    fn filters(&self) -> Vec<&Filter> {
        let mut result: Vec<&Filter> = self.filters.iter().collect();
        if let Some(builder) = &self.role_builder {
            result.append(&mut builder.filters());
        }
        result
    }
    fn order_part(&self) -> String {
        self.order.clone()
    }
}
#[allow(non_camel_case_types)]
pub struct ScreenBuilder_id {
    pub builder: ScreenBuilder,
}
impl ScreenBuilder_id {
    pub fn eq(&self, value: i64) -> ScreenBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "screens".to_string(),
            name: stringify!(id).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        ScreenBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i64>) -> ScreenBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "screens".to_string(),
            name: stringify!(id).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        ScreenBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct ScreenBuilder_role_id {
    pub builder: ScreenBuilder,
}
impl ScreenBuilder_role_id {
    pub fn eq(&self, value: i64) -> ScreenBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "screens".to_string(),
            name: stringify!(role_id).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        ScreenBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i64>) -> ScreenBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "screens".to_string(),
            name: stringify!(role_id).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        ScreenBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct ScreenBuilder_name {
    pub builder: ScreenBuilder,
}
impl ScreenBuilder_name {
    pub fn eq(&self, value: String) -> ScreenBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "screens".to_string(),
            name: stringify!(name).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        ScreenBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<String>) -> ScreenBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "screens".to_string(),
            name: stringify!(name).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        ScreenBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
