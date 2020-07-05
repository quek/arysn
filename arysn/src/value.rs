use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    I64(i64),
    String(String),
    DateTime(DateTime<Local>),
}

impl Value {
    pub fn to_sql_value(&self) -> String {
        match self {
            Self::Bool(x) => if *x { "TRUE" } else { "FALSE" }.to_string(),
            Self::I64(x) => x.to_string(),
            Self::String(x) => format!("'{}'", x.replace("'", "''")),
            Self::DateTime(x) => x.to_rfc3339(),
        }
    }
}

impl From<i64> for Value {
    fn from(x: i64) -> Self {
        Self::I64(x)
    }
}

impl From<String> for Value {
    fn from(x: String) -> Self {
        Self::String(x)
    }
}
