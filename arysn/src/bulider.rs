use crate::filter::Filter;
use crate::value::Value;
use async_trait::async_trait;

#[async_trait]
pub trait BuilderTrait {
    fn select(&self) -> String;
    fn from(&self) -> String;
    fn filters(&self) -> Vec<&Filter>;

    fn select_params(&self) -> Vec<&(dyn tokio_postgres::types::ToSql + Sync)> {
        let mut result: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        for filter in self.filters().iter() {
            match &filter.value {
                Value::Bool(x) => {
                    result.push(x);
                }
                Value::I64(x) => {
                    result.push(x);
                }
                Value::I32(x) => {
                    result.push(x);
                }
                Value::String(x) => {
                    result.push(x);
                }
                Value::DateTime(x) => {
                    result.push(x);
                }
                Value::VecBool(xs) => {
                    for x in xs.iter() {
                        result.push(x);
                    }
                }
                Value::VecI32(xs) => {
                    for x in xs.iter() {
                        result.push(x);
                    }
                }
                Value::VecI64(xs) => {
                    for x in xs.iter() {
                        result.push(x);
                    }
                }
                Value::VecString(xs) => {
                    for x in xs.iter() {
                        result.push(x);
                    }
                }
                Value::VecDateTime(xs) => {
                    for x in xs.iter() {
                        result.push(x);
                    }
                }
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
        format!(
            "SELECT {}.* FROM {} WHERE {}",
            self.select(),
            self.from(),
            filters.join(" AND ")
        )
    }
}
