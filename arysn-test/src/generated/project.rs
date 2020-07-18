use super::contribution::{Contribution, ContributionBuilder};
use arysn::prelude::*;
use async_recursion::async_recursion;
#[derive(Clone, Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub contributions: Option<Vec<Contribution>>,
}
#[derive(Clone, Debug)]
pub struct ProjectNew {
    pub id: Option<i64>,
    pub name: String,
}
impl Project {
    pub fn select() -> ProjectBuilder {
        ProjectBuilder {
            from: "projects".to_string(),
            ..ProjectBuilder::default()
        }
    }
    pub async fn delete(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .execute("DELETE FROM projects WHERE id = $1", &[&self.id])
            .await?;
        Ok(())
    }
    pub async fn update(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .execute(
                "UPDATE projects SET name = $1 WHERE id = $2",
                &[&self.name, &self.id],
            )
            .await?;
        Ok(())
    }
}
impl ProjectNew {
    pub async fn insert(&self, client: &tokio_postgres::Client) -> anyhow::Result<Project> {
        let mut target_columns: Vec<&str> = vec![];
        target_columns.push(stringify!(name));
        let target_columns = target_columns.join(", ");
        let mut bind_count: i32 = 0;
        bind_count += 1;
        let binds = (1..=bind_count)
            .map(|i| format!("${}", i))
            .collect::<Vec<_>>()
            .join(", ");
        let statement = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            "projects", target_columns, binds
        );
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        params.push(&self.name);
        let row = client.query_one(statement.as_str(), &params[..]).await?;
        Ok(row.into())
    }
}
impl From<tokio_postgres::row::Row> for Project {
    fn from(row: tokio_postgres::row::Row) -> Self {
        Self {
            id: row.get(0usize),
            name: row.get(1usize),
            contributions: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct ProjectBuilder {
    pub from: String,
    pub filters: Vec<Filter>,
    pub preload: bool,
    pub order: String,
    pub contributions_builder: Option<Box<ContributionBuilder>>,
}
impl ProjectBuilder {
    pub fn id(&self) -> ProjectBuilder_id {
        ProjectBuilder_id {
            builder: self.clone(),
        }
    }
    pub fn name(&self) -> ProjectBuilder_name {
        ProjectBuilder_name {
            builder: self.clone(),
        }
    }
    pub fn contributions<F>(&self, f: F) -> ProjectBuilder
    where
        F: FnOnce(&ContributionBuilder) -> ContributionBuilder,
    {
        ProjectBuilder {
            contributions_builder: Some(Box::new(f(self
                .contributions_builder
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
    pub async fn first(&self, client: &tokio_postgres::Client) -> anyhow::Result<Project> {
        let params = self.select_params();
        let row = client
            .query_one(self.select_sql().as_str(), &params[..])
            .await?;
        let x: Project = Project::from(row);
        Ok(x)
    }
    #[async_recursion]
    pub async fn load(&self, client: &tokio_postgres::Client) -> anyhow::Result<Vec<Project>> {
        let params = self.select_params();
        let rows = client
            .query(self.select_sql().as_str(), &params[..])
            .await?;
        let mut result: Vec<Project> = rows.into_iter().map(|row| Project::from(row)).collect();
        if let Some(builder) = &self.contributions_builder {
            if builder.preload {
                let ids = result.iter().map(|x| x.id).collect::<Vec<_>>();
                let children_builder = Contribution::select().project_id().eq_any(ids);
                let children_builder = ContributionBuilder {
                    from: children_builder.from,
                    filters: children_builder.filters,
                    ..(**builder).clone()
                };
                let children = children_builder.load(client).await?;
                result.iter_mut().for_each(|x| {
                    let mut ys = vec![];
                    for child in children.iter() {
                        if x.id == child.project_id {
                            ys.push(child.clone());
                        }
                    }
                    x.contributions = Some(ys);
                });
            }
        }
        Ok(result)
    }
}
impl BuilderTrait for ProjectBuilder {
    fn select(&self) -> String {
        "projects".to_string()
    }
    fn from(&self) -> String {
        let mut result: Vec<String> = vec![self.from.clone()];
        self.join(&mut result);
        result.join(" ")
    }
    fn join(&self, join_parts: &mut Vec<String>) {
        if let Some(builder) = &self.contributions_builder {
            join_parts.push(
                "INNER JOIN contributions ON contributions.project_id = projects.id".to_string(),
            );
            builder.join(join_parts);
        }
    }
    fn filters(&self) -> Vec<&Filter> {
        let mut result: Vec<&Filter> = self.filters.iter().collect();
        if let Some(builder) = &self.contributions_builder {
            result.append(&mut builder.filters());
        }
        result
    }
    fn order_part(&self) -> String {
        self.order.clone()
    }
}
#[allow(non_camel_case_types)]
pub struct ProjectBuilder_id {
    pub builder: ProjectBuilder,
}
impl ProjectBuilder_id {
    pub fn eq(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "projects".to_string(),
            name: stringify!(id).to_string(),
            value: vec![Box::new(value)],
            operator: "=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut v: Vec<Box<dyn ToSqlValue>> = vec![];
        for x in value {
            v.push(Box::new(x));
        }
        filters.push(Filter {
            table: "projects".to_string(),
            name: stringify!(id).to_string(),
            value: v,
            operator: "in".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct ProjectBuilder_name {
    pub builder: ProjectBuilder,
}
impl ProjectBuilder_name {
    pub fn eq(&self, value: String) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "projects".to_string(),
            name: stringify!(name).to_string(),
            value: vec![Box::new(value)],
            operator: "=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<String>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut v: Vec<Box<dyn ToSqlValue>> = vec![];
        for x in value {
            v.push(Box::new(x));
        }
        filters.push(Filter {
            table: "projects".to_string(),
            name: stringify!(name).to_string(),
            value: v,
            operator: "in".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
