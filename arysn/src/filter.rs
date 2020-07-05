use crate::value::Value;

#[derive(Clone, Debug)]
pub struct Filter {
    pub table: String,
    pub name: String,
    pub value: Value,
}

impl Filter {
    pub fn to_sql(&self) -> String {
        format!(
            "{}.{} = {}",
            &self.table,
            &self.name,
            self.value.to_sql_value()
        )
    }
}
