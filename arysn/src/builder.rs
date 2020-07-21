use crate::filter::Filter;
use tokio_postgres::types::ToSql;
use crate::order_item::OrderItem;

// TODO カラム名がメソッドとしてはえるので名前衝突しないように名前空間がわけたい
pub trait BuilderTrait {
    fn select(&self) -> String;
    fn from(&self) -> String;
    fn join(&self, join_parts: &mut Vec<String>);
    fn filters(&self) -> Vec<&Filter>;
    fn order(&self) -> &Vec<OrderItem>;
    fn limit(&self) -> Option<usize>;
    fn offset(&self) -> Option<usize>;

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
        let orders: &Vec<OrderItem> = BuilderTrait::order(self);
        let order_part = if orders.is_empty() {
            "".to_string()
        } else {
            format!(" ORDER BY {}", orders.iter().map(|x| x.to_sql()).collect::<Vec<_>>().join(", "))
        };
        let limit = match Self::limit(self) {
            Some(limit) => format!(" LIMIT {}", limit),
            _ => "".to_string(),
        };
        let offset = match Self::offset(self) {
            Some(offset) => format!(" OFFSET {}", offset),
            _ => "".to_string(),
        };
        // TODO 無条件に DISTINCT 付けるのはどうかと思う
        format!(
            "SELECT DISTINCT {}.* FROM {}{}{}{}{}",
            self.select(),
            self.from(),
            where_part,
            order_part,
            limit,
            offset
        )
    }
}
