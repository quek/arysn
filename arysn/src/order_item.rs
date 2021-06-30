#[derive(Clone, Debug)]
pub struct OrderItem {
    pub table: String,
    pub field: &'static str,
    pub asc_or_desc: &'static str,
}

impl OrderItem {
    pub fn to_sql(&self) -> String {
        if self.table.is_empty() {
            // order().by_string_literal_asc/desc("...")
            format!("{} {}", self.field, self.asc_or_desc)
        } else {
            format!("{}.{} {}", &self.table, self.field, self.asc_or_desc)
        }
    }
}
