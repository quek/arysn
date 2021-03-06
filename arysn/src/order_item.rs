#[derive(Clone, Debug)]
pub struct OrderItem {
    pub table: String,
    pub field: &'static str,
    pub asc_or_desc: &'static str,
}

impl OrderItem {
    pub fn to_sql(&self) -> String {
        format!("{}.{} {}", &self.table, self.field, self.asc_or_desc)
    }
}
