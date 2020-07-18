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
            "in" => {
                let len = self.values.len();
                if len == 0 {
                    ("1 = 2".to_string(), 0)
                } else {
                    (
                        format!(
                            "{}.{} in ({})",
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
