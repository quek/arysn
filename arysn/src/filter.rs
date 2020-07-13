use crate::value::Value;

#[derive(Clone, Debug)]
pub struct Filter {
    pub table: String,
    pub name: String,
    pub value: Value,
    pub operator: String,
}

impl Filter {
    pub fn to_sql(&self, bind_index: usize) -> (String, usize) {
        match self.operator.as_str() {
            "in" => match &self.value {
                Value::VecI64(x) => {
                    if x.is_empty() {
                        ("1 = 2".to_string(), 0)
                    } else {
                        (
                            format!(
                                "{}.{} in ({})",
                                &self.table,
                                &self.name,
                                (1..=x.len())
                                    .map(|i| format!("${}", i))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            x.len(),
                        )
                    }
                }
                _ => ("1 = 2".to_string(), 0),
            },
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
