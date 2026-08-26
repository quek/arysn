use crate::filter::Filter;
use crate::order_item::OrderItem;
use crate::prelude::BuilderAccessor;
use dyn_clone::DynClone;
use tokio_postgres::types::ToSql;

// TODO カラム名がメソッドとしてはえるので名前衝突しないように名前空間がわけたい
dyn_clone::clone_trait_object!(BuilderTrait);
pub trait BuilderTrait: BuilderAccessor + DynClone + Sync + Send {
    fn all_columns(&self) -> Vec<&'static str>;
    fn select(&self) -> String;
    fn from(&self, index: usize) -> (String, usize);
    fn join(&self, join_parts: &mut Vec<String>, index: usize) -> usize;
    fn query_filters(&self) -> Vec<&Filter>;
    fn group_by(&self) -> Option<&'static str>;
    fn order(&self) -> &Vec<OrderItem>;
    fn limit(&self) -> Option<usize>;
    fn offset(&self) -> Option<usize>;
    // 生成された builder が実装する。既存の実装を壊さないようにデフォルトを持たせている。
    fn for_update(&self) -> bool {
        false
    }

    fn where_part(&self, index: usize) -> String {
        let mut index = index;
        let mut filters: Vec<String> = vec![];
        for filter in self.query_filters().iter() {
            let (s, index_delta) = filter.to_sql(index);
            filters.push(s);
            index += index_delta;
        }
        if filters.is_empty() {
            "".to_string()
        } else {
            format!(
                " WHERE {}",
                filters
                    .join(" AND ")
                    .replace(" AND OR AND ", " OR ")
                    .replace("( AND ", "(")
                    .replace(" AND )", ")")
            )
        }
    }

    fn group_by_part(&self) -> String {
        if let Some(group_by) = self.group_by() {
            format!(" GROUP BY {}", group_by)
        } else {
            "".to_string()
        }
    }

    /// SELECT COUNT(...), MAX(...) などの集約関数用の SQL を組み立てる。
    fn aggregate(&self, select_part: String) -> (String, Vec<&(dyn ToSql + Sync)>) {
        let index: usize = 1;
        let (from_part, index_delta) = BuilderTrait::from(self, index);
        let sql = format!(
            "SELECT {} FROM {}{}{}",
            select_part,
            from_part,
            self.where_part(index + index_delta),
            self.group_by_part()
        );

        let params: Vec<&(dyn ToSql + Sync)> = self.select_params();

        (sql, params)
    }

    fn count(&self) -> (String, Vec<&(dyn ToSql + Sync)>) {
        self.aggregate(format!("COUNT(DISTINCT {}.*)", self.select()))
    }

    fn max(&self, column: &str) -> (String, Vec<&(dyn ToSql + Sync)>) {
        self.aggregate(format!("MAX({})", column))
    }

    fn min(&self, column: &str) -> (String, Vec<&(dyn ToSql + Sync)>) {
        self.aggregate(format!("MIN({})", column))
    }

    fn select_params(&self) -> Vec<&(dyn ToSql + Sync)> {
        let mut result: Vec<&(dyn ToSql + Sync)> = vec![];
        for join_select in self.join_selects().iter() {
            result.append(&mut join_select.builder.select_params());
        }
        for filter in self.query_filters().iter() {
            match filter {
                Filter::Column(column) => {
                    for value in column.values.iter() {
                        result.push(value.as_to_sql().unwrap());
                    }
                }
                Filter::Builder(builder) => {
                    result.append(&mut builder.select_params());
                }
            }
        }
        result
    }

    fn select_sql(&self) -> String {
        let orders: &Vec<OrderItem> = BuilderTrait::order(self);
        let select = BuilderTrait::all_columns(self)
            .iter()
            .map(|column| format!("{}.{}", BuilderTrait::select(self), column))
            .chain(
                orders
                    .iter()
                    .filter(|x| x.table.is_empty())
                    .map(|x| x.field.to_string()),
            )
            .collect::<Vec<_>>()
            .join(", ");
        let index: usize = 1;
        let (from_part, index_delta) = BuilderTrait::from(self, index);
        let where_part = self.where_part(index + index_delta);
        let group_by_part = self.group_by_part();
        let order_part = if orders.is_empty() {
            "".to_string()
        } else {
            format!(
                " ORDER BY {}",
                orders
                    .iter()
                    .map(|x| x.to_sql())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let limit = match Self::limit(self) {
            Some(limit) => format!(" LIMIT {}", limit),
            _ => "".to_string(),
        };
        let offset = match Self::offset(self) {
            Some(offset) => format!(" OFFSET {}", offset),
            _ => "".to_string(),
        };
        // FOR UPDATE は DISTINCT と併用できない
        let (distinct, for_update) = if Self::for_update(self) {
            ("", " FOR UPDATE")
        } else {
            ("DISTINCT ", "")
        };
        // TODO 無条件に DISTINCT 付けるのはどうかと思う
        format!(
            "SELECT {}{} FROM {}{}{}{}{}{}{}",
            distinct, select, from_part, where_part, group_by_part, order_part, limit, offset,
            for_update
        )
    }
}

use core::fmt::Debug;
impl Debug for dyn BuilderTrait {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "BuilderTrait {{ table_name: {:?}, table_name_as: {:?}, filters: {:?}, preload: {:?} }}",
            self.table_name(),
            self.table_name_as(),
            self.filters(),
            self.preload(),
        )
    }
}
