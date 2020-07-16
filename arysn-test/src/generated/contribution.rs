use super::project::{Project, ProjectBuilder};
use super::user::{User, UserBuilder};
use arysn::prelude::*;
use async_recursion::async_recursion;
#[derive(Clone, Debug)]
pub struct Contribution {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub project: Option<Project>,
    pub user: Option<User>,
}
#[derive(Clone, Debug)]
pub struct ContributionNew {
    pub id: Option<i64>,
    pub project_id: i64,
    pub user_id: i64,
}
impl Contribution {
    pub fn select() -> ContributionBuilder {
        ContributionBuilder {
            from: "contributions".to_string(),
            ..ContributionBuilder::default()
        }
    }
    pub async fn delete(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .execute("DELETE FROM contributions WHERE id = $1", &[&self.id])
            .await?;
        Ok(())
    }
    pub async fn update(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .execute(
                "UPDATE contributions SET project_id = $1, user_id = $2 WHERE id = $3",
                &[&self.project_id, &self.user_id, &self.id],
            )
            .await?;
        Ok(())
    }
}
impl ContributionNew {
    pub async fn insert(&self, client: &tokio_postgres::Client) -> anyhow::Result<Contribution> {
        let mut target_columns: Vec<&str> = vec![];
        target_columns.push(stringify!(project_id));
        target_columns.push(stringify!(user_id));
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
            "contributions", target_columns, binds
        );
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        params.push(&self.project_id);
        params.push(&self.user_id);
        let row = client.query_one(statement.as_str(), &params[..]).await?;
        Ok(row.into())
    }
}
impl From<tokio_postgres::row::Row> for Contribution {
    fn from(row: tokio_postgres::row::Row) -> Self {
        Self {
            id: row.get(0usize),
            project_id: row.get(1usize),
            user_id: row.get(2usize),
            project: None,
            user: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct ContributionBuilder {
    pub from: String,
    pub filters: Vec<Filter>,
    pub preload: bool,
    pub project_builder: Option<Box<ProjectBuilder>>,
    pub user_builder: Option<Box<UserBuilder>>,
}
impl ContributionBuilder {
    pub fn id(&self) -> ContributionBuilder_id {
        ContributionBuilder_id {
            builder: self.clone(),
        }
    }
    pub fn project_id(&self) -> ContributionBuilder_project_id {
        ContributionBuilder_project_id {
            builder: self.clone(),
        }
    }
    pub fn user_id(&self) -> ContributionBuilder_user_id {
        ContributionBuilder_user_id {
            builder: self.clone(),
        }
    }
    pub fn project<F>(&self, f: F) -> ContributionBuilder
    where
        F: FnOnce(&ProjectBuilder) -> ProjectBuilder,
    {
        ContributionBuilder {
            project_builder: Some(Box::new(f(self
                .project_builder
                .as_ref()
                .unwrap_or(&Default::default())))),
            ..self.clone()
        }
    }
    pub fn user<F>(&self, f: F) -> ContributionBuilder
    where
        F: FnOnce(&UserBuilder) -> UserBuilder,
    {
        ContributionBuilder {
            user_builder: Some(Box::new(f(self
                .user_builder
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
    pub async fn first(&self, client: &tokio_postgres::Client) -> anyhow::Result<Contribution> {
        let params = self.select_params();
        let row = client
            .query_one(self.select_sql().as_str(), &params[..])
            .await?;
        let x: Contribution = Contribution::from(row);
        Ok(x)
    }
    #[async_recursion]
    pub async fn load(&self, client: &tokio_postgres::Client) -> anyhow::Result<Vec<Contribution>> {
        let params = self.select_params();
        let rows = client
            .query(self.select_sql().as_str(), &params[..])
            .await?;
        let mut result: Vec<Contribution> = rows
            .into_iter()
            .map(|row| Contribution::from(row))
            .collect();
        if let Some(builder) = &self.project_builder {
            if builder.preload {
                let ids = result.iter().map(|x| x.project_id).collect::<Vec<_>>();
                let parents_builder = Project::select().id().eq_any(ids);
                let parents_builder = ProjectBuilder {
                    from: parents_builder.from,
                    filters: parents_builder.filters,
                    ..(**builder).clone()
                };
                let parents = parents_builder.load(client).await?;
                result.iter_mut().for_each(|x| {
                    for parent in parents.iter() {
                        if x.project_id == parent.id {
                            x.project = Some(parent.clone());
                            break;
                        }
                    }
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
impl BuilderTrait for ContributionBuilder {
    fn select(&self) -> String {
        "contributions".to_string()
    }
    fn from(&self) -> String {
        let mut result: Vec<String> = vec![self.from.clone()];
        self.join(&mut result);
        result.join(" ")
    }
    fn join(&self, join_parts: &mut Vec<String>) {
        if let Some(builder) = &self.project_builder {
            join_parts
                .push("INNER JOIN projects ON projects.id = contributions.project_id".to_string());
            builder.join(join_parts);
        }
        if let Some(builder) = &self.user_builder {
            join_parts.push("INNER JOIN users ON users.id = contributions.user_id".to_string());
            builder.join(join_parts);
        }
    }
    fn filters(&self) -> Vec<&Filter> {
        let mut result: Vec<&Filter> = self.filters.iter().collect();
        if let Some(builder) = &self.project_builder {
            result.append(&mut builder.filters());
        }
        if let Some(builder) = &self.user_builder {
            result.append(&mut builder.filters());
        }
        result
    }
}
#[allow(non_camel_case_types)]
pub struct ContributionBuilder_id {
    pub builder: ContributionBuilder,
}
impl ContributionBuilder_id {
    pub fn eq(&self, value: i64) -> ContributionBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "contributions".to_string(),
            name: stringify!(id).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        ContributionBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i64>) -> ContributionBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "contributions".to_string(),
            name: stringify!(id).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        ContributionBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct ContributionBuilder_project_id {
    pub builder: ContributionBuilder,
}
impl ContributionBuilder_project_id {
    pub fn eq(&self, value: i64) -> ContributionBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "contributions".to_string(),
            name: stringify!(project_id).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        ContributionBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i64>) -> ContributionBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "contributions".to_string(),
            name: stringify!(project_id).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        ContributionBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct ContributionBuilder_user_id {
    pub builder: ContributionBuilder,
}
impl ContributionBuilder_user_id {
    pub fn eq(&self, value: i64) -> ContributionBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "contributions".to_string(),
            name: stringify!(user_id).to_string(),
            value: value.into(),
            operator: "=".to_string(),
        });
        ContributionBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, value: Vec<i64>) -> ContributionBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: "contributions".to_string(),
            name: stringify!(user_id).to_string(),
            value: value.into(),
            operator: "in".to_string(),
        });
        ContributionBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
