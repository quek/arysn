use super::contribution::Contribution;
use super::contribution_impl::ContributionBuilder;
use super::project::*;
use super::user::User;
use super::user_impl::UserBuilder;
use arysn::prelude::*;
use async_recursion::async_recursion;
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
        client . execute ( "UPDATE projects SET name = $1, parent_project_id = $2, create_user_id = $3, update_user_id = $4 WHERE id = $5" , & [ & self . name , & self . parent_project_id , & self . create_user_id , & self . update_user_id , & self . id ] ) . await ? ;
        Ok(())
    }
}
impl ProjectNew {
    pub async fn insert(&self, client: &tokio_postgres::Client) -> anyhow::Result<Project> {
        let mut target_columns: Vec<&str> = vec![];
        target_columns.push(stringify!(name));
        target_columns.push(stringify!(parent_project_id));
        target_columns.push(stringify!(create_user_id));
        target_columns.push(stringify!(update_user_id));
        let target_columns = target_columns.join(", ");
        let mut bind_count: i32 = 0;
        bind_count += 1;
        bind_count += 1;
        bind_count += 1;
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
        params.push(&self.parent_project_id);
        params.push(&self.create_user_id);
        params.push(&self.update_user_id);
        let row = client.query_one(statement.as_str(), &params[..]).await?;
        Ok(row.into())
    }
}
impl From<tokio_postgres::row::Row> for Project {
    fn from(row: tokio_postgres::row::Row) -> Self {
        Self {
            id: row.get(0usize),
            name: row.get(1usize),
            parent_project_id: row.get(2usize),
            create_user_id: row.get(3usize),
            update_user_id: row.get(4usize),
            contributions: None,
            create_user: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct ProjectBuilder {
    pub from: String,
    pub table_name_as: Option<String>,
    pub filters: Vec<Filter>,
    pub preload: bool,
    pub orders: Vec<OrderItem>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub contributions_builder: Option<Box<ContributionBuilder>>,
    pub create_user_builder: Option<Box<UserBuilder>>,
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
    pub fn parent_project_id(&self) -> ProjectBuilder_parent_project_id {
        ProjectBuilder_parent_project_id {
            builder: self.clone(),
        }
    }
    pub fn create_user_id(&self) -> ProjectBuilder_create_user_id {
        ProjectBuilder_create_user_id {
            builder: self.clone(),
        }
    }
    pub fn update_user_id(&self) -> ProjectBuilder_update_user_id {
        ProjectBuilder_update_user_id {
            builder: self.clone(),
        }
    }
    pub fn contributions<F>(&self, f: F) -> ProjectBuilder
    where
        F: FnOnce(&ContributionBuilder) -> ContributionBuilder,
    {
        let mut child_builder = f(self
            .contributions_builder
            .as_ref()
            .unwrap_or(&Default::default()));
        let mut builder = self.clone();
        builder.orders.append(&mut child_builder.orders);
        builder.contributions_builder = Some(Box::new(child_builder));
        builder
    }
    pub fn create_user<F>(&self, f: F) -> ProjectBuilder
    where
        F: FnOnce(&UserBuilder) -> UserBuilder,
    {
        ProjectBuilder {
            create_user_builder: Some(Box::new(f(self.create_user_builder.as_ref().unwrap_or(
                &Box::new(UserBuilder {
                    table_name_as: Some("create_user".to_string()),
                    ..Default::default()
                }),
            )))),
            ..self.clone()
        }
    }
    pub fn limit(&self, value: usize) -> Self {
        Self {
            limit: Some(value),
            ..self.clone()
        }
    }
    pub fn offset(&self, value: usize) -> Self {
        Self {
            offset: Some(value),
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
        if let Some(builder) = &self.create_user_builder {
            if builder.preload {
                let ids = result.iter().map(|x| x.create_user_id).collect::<Vec<_>>();
                let parents_builder = User::select().id().eq_any(ids);
                let parents_builder = UserBuilder {
                    from: parents_builder.from,
                    filters: parents_builder.filters,
                    ..(**builder).clone()
                };
                let parents = parents_builder.load(client).await?;
                result.iter_mut().for_each(|x| {
                    for parent in parents.iter() {
                        if x.create_user_id == parent.id {
                            x.create_user = Some(parent.clone());
                            break;
                        }
                    }
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
        if let Some(builder) = &self.create_user_builder {
            join_parts.push(
                "INNER JOIN users AS create_user ON create_user.id = projects.create_user_id"
                    .to_string(),
            );
            builder.join(join_parts);
        }
    }
    fn filters(&self) -> Vec<&Filter> {
        let mut result: Vec<&Filter> = self.filters.iter().collect();
        if let Some(builder) = &self.contributions_builder {
            result.append(&mut builder.filters());
        }
        if let Some(builder) = &self.create_user_builder {
            result.append(&mut builder.filters());
        }
        result
    }
    fn order(&self) -> &Vec<OrderItem> {
        &self.orders
    }
    fn limit(&self) -> Option<usize> {
        self.limit
    }
    fn offset(&self) -> Option<usize> {
        self.offset
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
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vec![Box::new(value)],
            operator: "=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gt(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vec![Box::new(value)],
            operator: ">".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lt(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vec![Box::new(value)],
            operator: "<".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gte(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vec![Box::new(value)],
            operator: ">=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lte(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vec![Box::new(value)],
            operator: "<=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_eq(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vec![Box::new(value)],
            operator: "<>".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vec![],
            operator: "IS NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_not_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vec![],
            operator: "IS NOT NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn between(&self, from: i64, to: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vec![Box::new(from), Box::new(to)],
            operator: "BETWEEN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn r#in(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_in(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(id).to_string(),
            values: vs,
            operator: "NOT IN".to_string(),
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
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vec![Box::new(value)],
            operator: "=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gt(&self, value: String) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vec![Box::new(value)],
            operator: ">".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lt(&self, value: String) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vec![Box::new(value)],
            operator: "<".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gte(&self, value: String) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vec![Box::new(value)],
            operator: ">=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lte(&self, value: String) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vec![Box::new(value)],
            operator: "<=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_eq(&self, value: String) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vec![Box::new(value)],
            operator: "<>".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vec![],
            operator: "IS NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_not_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vec![],
            operator: "IS NOT NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn between(&self, from: String, to: String) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vec![Box::new(from), Box::new(to)],
            operator: "BETWEEN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, values: Vec<String>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn r#in(&self, values: Vec<String>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_in(&self, values: Vec<String>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(name).to_string(),
            values: vs,
            operator: "NOT IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct ProjectBuilder_parent_project_id {
    pub builder: ProjectBuilder,
}
impl ProjectBuilder_parent_project_id {
    pub fn eq(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vec![Box::new(value)],
            operator: "=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gt(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vec![Box::new(value)],
            operator: ">".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lt(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vec![Box::new(value)],
            operator: "<".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gte(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vec![Box::new(value)],
            operator: ">=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lte(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vec![Box::new(value)],
            operator: "<=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_eq(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vec![Box::new(value)],
            operator: "<>".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vec![],
            operator: "IS NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_not_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vec![],
            operator: "IS NOT NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn between(&self, from: i64, to: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vec![Box::new(from), Box::new(to)],
            operator: "BETWEEN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn r#in(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_in(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(parent_project_id).to_string(),
            values: vs,
            operator: "NOT IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct ProjectBuilder_create_user_id {
    pub builder: ProjectBuilder,
}
impl ProjectBuilder_create_user_id {
    pub fn eq(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: "=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gt(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: ">".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lt(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: "<".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gte(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: ">=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lte(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: "<=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_eq(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: "<>".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vec![],
            operator: "IS NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_not_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vec![],
            operator: "IS NOT NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn between(&self, from: i64, to: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vec![Box::new(from), Box::new(to)],
            operator: "BETWEEN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn r#in(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_in(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(create_user_id).to_string(),
            values: vs,
            operator: "NOT IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
#[allow(non_camel_case_types)]
pub struct ProjectBuilder_update_user_id {
    pub builder: ProjectBuilder,
}
impl ProjectBuilder_update_user_id {
    pub fn eq(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: "=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gt(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: ">".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lt(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: "<".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn gte(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: ">=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn lte(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: "<=".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_eq(&self, value: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vec![Box::new(value)],
            operator: "<>".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vec![],
            operator: "IS NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn is_not_null(&self) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vec![],
            operator: "IS NOT NULL".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn between(&self, from: i64, to: i64) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vec![Box::new(from), Box::new(to)],
            operator: "BETWEEN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn eq_any(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn r#in(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vs,
            operator: "IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
    pub fn not_in(&self, values: Vec<i64>) -> ProjectBuilder {
        let mut filters = self.builder.filters.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        filters.push(Filter {
            table: self
                .builder
                .table_name_as
                .as_ref()
                .unwrap_or(&"projects".to_string())
                .to_string(),
            name: stringify!(update_user_id).to_string(),
            values: vs,
            operator: "NOT IN".to_string(),
        });
        ProjectBuilder {
            filters,
            ..self.builder.clone()
        }
    }
}
impl ProjectBuilder {
    pub fn order(&self) -> ProjectOrderBuilder {
        ProjectOrderBuilder {
            builder: self.clone(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct ProjectOrderBuilder {
    pub builder: ProjectBuilder,
}
impl ProjectOrderBuilder {
    pub fn id(&self) -> ProjectOrderAscOrDesc {
        ProjectOrderAscOrDesc {
            field: "id",
            order_builder: self.clone(),
        }
    }
    pub fn name(&self) -> ProjectOrderAscOrDesc {
        ProjectOrderAscOrDesc {
            field: "name",
            order_builder: self.clone(),
        }
    }
    pub fn parent_project_id(&self) -> ProjectOrderAscOrDesc {
        ProjectOrderAscOrDesc {
            field: "parent_project_id",
            order_builder: self.clone(),
        }
    }
    pub fn create_user_id(&self) -> ProjectOrderAscOrDesc {
        ProjectOrderAscOrDesc {
            field: "create_user_id",
            order_builder: self.clone(),
        }
    }
    pub fn update_user_id(&self) -> ProjectOrderAscOrDesc {
        ProjectOrderAscOrDesc {
            field: "update_user_id",
            order_builder: self.clone(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct ProjectOrderAscOrDesc {
    pub field: &'static str,
    pub order_builder: ProjectOrderBuilder,
}
impl ProjectOrderAscOrDesc {
    pub fn asc(&self) -> ProjectBuilder {
        let mut builder = self.order_builder.builder.clone();
        builder.orders.push(OrderItem {
            field: self.field,
            asc_or_desc: "ASC",
        });
        builder
    }
    pub fn desc(&self) -> ProjectBuilder {
        let mut builder = self.order_builder.builder.clone();
        builder.orders.push(OrderItem {
            field: self.field,
            asc_or_desc: "DESC",
        });
        builder
    }
}
