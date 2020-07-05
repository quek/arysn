use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    I32(i32),
    I64(i64),
    String(String),
    DateTime(DateTime<Local>),
}

impl Value {
    pub fn to_sql_value(&self) -> String {
        match self {
            Self::Bool(x) => if *x { "TRUE" } else { "FALSE" }.to_string(),
            Self::I64(x) => x.to_string(),
            Self::I32(x) => x.to_string(),
            Self::String(x) => format!("'{}'", x.replace("'", "''")),
            Self::DateTime(x) => x.format("'%Y-%m-%d %H:%M:%S%.6f %:z'").to_string(),
        }
    }

    pub fn to_sql(&self) -> &(dyn tokio_postgres::types::ToSql + Sync) {
        match self {
            Self::Bool(x) => x,
            Self::I64(x) => x,
            Self::I32(x) => x,
            Self::String(x) => x,
            Self::DateTime(x) => x,
        }
    }
}

impl From<bool> for Value {
    fn from(x: bool) -> Self {
        Self::Bool(x)
    }
}

impl From<i32> for Value {
    fn from(x: i32) -> Self {
        Self::I32(x)
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

impl From<DateTime<Local>> for Value {
    fn from(x: DateTime<Local>) -> Self {
        Self::DateTime(x)
    }
}
