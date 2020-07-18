use crate::filter::Filter;
use tokio_postgres::types::ToSql;

// TODO カラム名がメソッドとしてはえるので名前衝突しないように名前空間がわけたい
pub trait BuilderTrait {
    fn select(&self) -> String;
    fn from(&self) -> String;
    fn join(&self, join_parts: &mut Vec<String>);
    fn filters(&self) -> Vec<&Filter>;
    fn order_part(&self) -> String;

    fn select_params(&self) -> Vec<&(dyn ToSql + Sync)> {
        let mut result: Vec<&(dyn ToSql + Sync)> = vec![];
        for filter in self.filters().iter() {
            for value in filter.values.iter() {
                result.push(value.as_to_sql().unwrap());
            }
        }
        result
    }

    fn select_sql(&self) -> String {
        let mut index: usize = 1;
        let mut filters: Vec<String> = vec![];
        for filter in self.filters().iter() {
            let (s, i) = filter.to_sql(index);
            filters.push(s);
            index += i;
        }
        let where_part = if filters.is_empty() {
            "".to_string()
        } else {
            format!(" WHERE {}", filters.join(" AND "))
        };
        let order_part = if self.order_part().is_empty() {
            "".to_string()
        } else {
            format!(" ORDER BY {}", &self.order_part())
        };
        // TODO 無条件に DISTINCT 付けるのはどうかと思う
        format!(
            "SELECT DISTINCT {}.* FROM {}{}{}",
            self.select(),
            self.from(),
            where_part,
            order_part,
        )
    }
}
