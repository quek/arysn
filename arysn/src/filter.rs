use crate::value::ToSqlValue;

#[derive(Clone, Debug)]
pub struct Filter {
    pub table: String,
    pub name: String,
    pub values: Vec<Box<dyn ToSqlValue>>,
    pub operator: &'static str,
    pub preload: bool,
}

impl Filter {
    pub fn to_sql(&self, bind_index: usize) -> (String, usize) {
        match self.operator {
            "(" => ("(".to_string(), 0),
            ")" => (")".to_string(), 0),
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
                            (0..len)
                                .map(|i| format!("${}", i + bind_index))
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
                            (0..len)
                                .map(|i| format!("${}", i + bind_index))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        len,
                    )
                }
            }
            "OR" => ("OR".to_string(), 0),
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
            _ => {
                if self.name.is_empty() {
                    (self.operator.to_string(), 0)
                } else {
                    (
                        format!(
                            "{}.{} {} ${}",
                            &self.table, &self.name, &self.operator, bind_index
                        ),
                        1,
                    )
                }
            }
        }
    }
}
