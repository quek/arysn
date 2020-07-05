use crate::value::Value;

#[derive(Clone, Debug)]
pub struct Filter {
    pub table: String,
    pub name: String,
    pub value: Value,
}

impl Filter {
    pub fn to_sql(&self, bind_index: usize) -> String {
        format!("{}.{} = ${}", &self.table, &self.name, bind_index)
    }
}
