use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use std::fmt::Debug;
use tokio_postgres::{Client, NoTls, Row};

#[derive(Clone, Debug)]
pub enum Value {
    I64(i64),
    String(String),
}

impl Value {
    pub fn to_sql_value(&self) -> String {
        match self {
            Self::I64(x) => x.to_string(),
            Self::String(x) => format!("'{}'", x.replace("'", "''")),
        }
    }
}

impl From<i64> for Value {
    fn from(x: i64) -> Self {
        Self::I64(x)
    }
}

impl From<String> for Value {
    fn from(x: String) -> Self {
        Self::String(x)
    }
}

pub trait FilterTrait {
    fn to_sql(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct Filter {
    pub table: String,
    pub name: String,
    pub value: Value,
}

impl Filter {
    pub fn to_sql_impl(&self) -> String {
        format!(
            "{}.{} = {}",
            &self.table,
            &self.name,
            self.value.to_sql_value()
        )
    }
}

impl FilterTrait for Filter {
    fn to_sql(&self) -> String {
        self.to_sql_impl()
    }
}

pub async fn connect() -> Result<Client> {
    debug!("connect");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    #[cfg(debug_assertions)]
    client.execute("SET log_statement = 'all'", &[]).await?;
    client.execute("SET TIME ZONE 'Japan'", &[]).await?;
    Ok(client)
}

#[async_trait]
pub trait BuilderTrait {
    fn filters(&self) -> &Vec<Filter>;

    async fn load<T>(&self, client: &Client) -> Result<Vec<T>>
    where
        T: From<Row>,
    {
        let rows = client.query(self.sql().as_str(), &[]).await?;
        let xs: Vec<T> = rows.into_iter().map(|row| T::from(row)).collect();
        Ok(xs)
    }

    fn sql(&self) -> String {
        format!(
            "SELECT * FROM users WHERE {}",
            self.filters()
                .iter()
                .map(|x| x.to_sql())
                .collect::<Vec<_>>()
                .join(" AND ")
        )
    }
}

pub trait BuilderColumnTrait {}

#[derive(Clone, Debug, Default)]
pub struct Builder {
    pub from: Option<String>,
    pub filter: Vec<String>,
    pub order: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use arysn_macro::defar;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    defar!(User {
        table_name: users,
        foo: bar
    });

    #[tokio::test]
    async fn it_works() -> Result<()> {
        init();

        let client = connect().await?;

        let users: Vec<User> = User::select()
            .id()
            .eq(1)
            .name()
            .eq("ユーザ1".to_string())
            .load(&client)
            .await?;
        assert_eq!(1, users.len());
        let user = &users[0];
        assert_eq!(1, user.id);
        assert_eq!("ユーザ1", user.name);
        assert_eq!(Some("旅人".to_string()), user.title);

        Ok(())
    }
}
