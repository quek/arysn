use crate::filter::Filter;
use anyhow::Result;
use async_trait::async_trait;
use tokio_postgres::{Client, Row};

#[async_trait]
pub trait BuilderTrait {
    fn from(&self) -> &String;
    fn filters(&self) -> &Vec<Filter>;

    async fn first_impl<T>(&self, client: &Client) -> Result<T>
    where
        T: From<Row>,
    {
        let row = client.query_one(self.select_sql().as_str(), &[]).await?;
        let x: T = T::from(row);
        Ok(x)
    }

    async fn load_impl<T>(&self, client: &Client) -> Result<Vec<T>>
    where
        T: From<Row>,
    {
        let rows = client.query(self.select_sql().as_str(), &[]).await?;
        let xs: Vec<T> = rows.into_iter().map(|row| T::from(row)).collect();
        Ok(xs)
    }

    fn select_sql(&self) -> String {
        format!(
            "SELECT * FROM {} WHERE {}",
            self.from(),
            self.filters()
                .iter()
                .map(|x| x.to_sql())
                .collect::<Vec<_>>()
                .join(" AND ")
        )
    }
}
