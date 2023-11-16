use crate::{prelude::BuilderTrait, value::ToSqlValue};

#[derive(Clone, Debug)]
pub enum Filter {
    Column(Column),
    Builder(Box<dyn BuilderTrait>),
}

#[derive(Clone, Debug)]
pub struct Column {
    pub table: String,
    pub name: String,
    pub values: Vec<Box<dyn ToSqlValue>>,
    pub operator: &'static str,
    pub preload: bool,
}

impl Filter {
    pub fn to_sql(&self, bind_index: usize) -> (String, usize) {
        match self {
            Filter::Column(column) => match column.operator {
                "(" => ("(".to_string(), 0),
                ")" => (")".to_string(), 0),
                "IN" => {
                    let len = column.values.len();
                    if len == 0 {
                        ("1 = 2".to_string(), 0)
                    } else {
                        (
                            format!(
                                "{}.{} IN ({})",
                                &column.table,
                                &column.name,
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
                    let len = column.values.len();
                    if len == 0 {
                        ("1 = 2".to_string(), 0)
                    } else {
                        (
                            format!(
                                "{}.{} NOT IN ({})",
                                &column.table,
                                &column.name,
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
                "IS NULL" => (format!("{}.{} IS NULL", &column.table, &column.name), 0),
                "IS NOT NULL" => (format!("{}.{} IS NOT NULL", &column.table, &column.name), 0),
                "BETWEEN" => (
                    format!(
                        "{}.{} {} ${} AND ${}",
                        &column.table,
                        &column.name,
                        &column.operator,
                        bind_index,
                        bind_index + 1
                    ),
                    2,
                ),
                "LITERAL" => {
                    let mut bind_index_delta = 0;
                    let re = regex::Regex::new(r"\$\d+").unwrap();
                    let sql = re.replace_all(&column.name, |caps: &regex::Captures| {
                        let n: usize = caps[0][1..].parse().unwrap();
                        bind_index_delta += 1;
                        format!("${}", n - 1 + bind_index)
                    });
                    (sql.to_string(), bind_index_delta)
                }
                _ => (
                    format!(
                        "{}.{} {} ${}",
                        &column.table, &column.name, &column.operator, bind_index
                    ),
                    1,
                ),
            },
            Filter::Builder(builder) => {
                let mut index: usize = bind_index;
                let mut filters: Vec<String> = vec![];
                for filter in builder.query_filters().iter() {
                    let (s, i) = filter.to_sql(index);
                    filters.push(s);
                    index += i;
                }
                (filters.join(" AND "), index - bind_index)
            }
        }
    }
}
