use crate::filter::Filter;
use anyhow::Result;
use async_trait::async_trait;
use tokio_postgres::{Client, Row};

#[async_trait]
pub trait BuilderTrait {
    fn from(&self) -> &String;
    fn filters(&self) -> &Vec<Filter>;

    async fn load_impl<T>(&self, client: &Client) -> Result<Vec<T>>
    where
        T: From<Row>,
    {
        let params = self.select_params();
        let rows = client
            .query(self.select_sql().as_str(), &params[..])
            .await?;
        let xs: Vec<T> = rows.into_iter().map(|row| T::from(row)).collect();
        Ok(xs)
    }

    fn select_params(&self) -> Vec<&(dyn tokio_postgres::types::ToSql + Sync)> {
        self.filters()
            .iter()
            .map(|filter| filter.value.to_sql())
            .collect::<Vec<_>>()
    }

    fn select_sql(&self) -> String {
        format!(
            "SELECT * FROM {} WHERE {}",
            self.from(),
            self.filters()
                .iter()
                .enumerate()
                .map(|(index, filter)| filter.to_sql(index + 1))
                .collect::<Vec<_>>()
                .join(" AND ")
        )
    }
}
