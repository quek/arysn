use crate::filter::Filter;
use anyhow::Result;
use async_trait::async_trait;
use tokio_postgres::{Client, Row};

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
