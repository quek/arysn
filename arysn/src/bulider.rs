use crate::filter::Filter;
use async_trait::async_trait;

#[async_trait]
pub trait BuilderTrait {
    fn select(&self) -> String;
    fn from(&self) -> String;
    fn filters(&self) -> Vec<&Filter>;

    fn select_params(&self) -> Vec<&(dyn tokio_postgres::types::ToSql + Sync)> {
        let filters = self.filters();
        filters
            .into_iter()
            .map(|filter| filter.value.to_sql())
            .collect::<Vec<_>>()
    }

    fn select_sql(&self) -> String {
        format!(
            "SELECT {}.* FROM {} WHERE {}",
            self.select(),
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
