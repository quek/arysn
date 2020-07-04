use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use tokio_postgres::{Client, NoTls, Row};

trait ToSqlValue {
    fn to_sql_value(&self) -> String;
}

impl ToSqlValue for i64 {
    fn to_sql_value(&self) -> String {
        format!("{}", self)
    }
}

impl ToSqlValue for String {
    fn to_sql_value(&self) -> String {
        format!("'{}'", self.replace("'", "''"))
    }
}

impl ToSqlValue for Option<String> {
    fn to_sql_value(&self) -> String {
        if let Some(x) = self {
            format!("'{}'", x)
        } else {
            "TODO".to_string()
        }
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
    fn filters(&self) -> &Vec<String>;

    async fn load<T>(&self, client: &Client) -> Result<Vec<T>>
    where
        T: From<Row>,
    {
        let rows = client.query(self.sql().as_str(), &[]).await?;
        let xs: Vec<T> = rows.into_iter().map(|row| T::from(row)).collect();
        Ok(xs)
    }

    fn sql(&self) -> String {
        format!("SELECT * FROM users WHERE {}", self.filters().join(" AND "))
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
