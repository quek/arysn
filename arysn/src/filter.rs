use crate::value::ToSqlValue;

#[derive(Clone, Debug)]
pub struct Filter {
    pub table: String,
    pub name: String,
    pub values: Vec<Box<dyn ToSqlValue>>,
    pub operator: String,
}

impl Filter {
    pub fn to_sql(&self, bind_index: usize) -> (String, usize) {
        match self.operator.as_str() {
            "IN" => {
                let len = self.values.len();
                if len == 0 {
                    ("1 = 2".to_string(), 0)
                } else {
                    (
                        format!(
                            "{}.{} IN ({})",
                            &self.table,
                            &self.name,
                            (1..=len)
                                .map(|i| format!("${}", i))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        len,
                    )
                }
            }
            "NOT IN" => {
                let len = self.values.len();
                if len == 0 {
                    ("1 = 2".to_string(), 0)
                } else {
                    (
                        format!(
                            "{}.{} NOT IN ({})",
                            &self.table,
                            &self.name,
                            (1..=len)
                                .map(|i| format!("${}", i))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        len,
                    )
                }
            }
            "IS NULL" => (format!("{}.{} IS NULL", &self.table, &self.name), 0),
            "IS NOT NULL" => (format!("{}.{} IS NOT NULL", &self.table, &self.name), 0),
            "BETWEEN" => (
                format!(
                    "{}.{} {} ${} AND ${}",
                    &self.table,
                    &self.name,
                    &self.operator,
                    bind_index,
                    bind_index + 1
                ),
                2,
            ),
            _ => (
                format!(
                    "{}.{} {} ${}",
                    &self.table, &self.name, &self.operator, bind_index
                ),
                1,
            ),
        }
    }
}
