use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    I32(i32),
    I64(i64),
    String(String),
    DateTime(DateTime<Local>),
    VecBool(Vec<bool>),
    VecI32(Vec<i32>),
    VecI64(Vec<i64>),
    VecString(Vec<String>),
    VecDateTime(Vec<DateTime<Local>>),
}

impl Value {
    pub fn to_sql(&self) -> &(dyn tokio_postgres::types::ToSql + Sync) {
        match self {
            Self::Bool(x) => x,
            Self::I64(x) => x,
            Self::I32(x) => x,
            Self::String(x) => x,
            Self::DateTime(x) => x,
            Self::VecBool(x) => x,
            Self::VecI32(x) => x,
            Self::VecI64(x) => x,
            Self::VecString(x) => x,
            Self::VecDateTime(x) => x,
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

impl From<Vec<bool>> for Value {
    fn from(x: Vec<bool>) -> Self {
        Self::VecBool(x)
    }
}

impl From<Vec<i32>> for Value {
    fn from(x: Vec<i32>) -> Self {
        Self::VecI32(x)
    }
}

impl From<Vec<i64>> for Value {
    fn from(x: Vec<i64>) -> Self {
        Self::VecI64(x)
    }
}

impl From<Vec<String>> for Value {
    fn from(x: Vec<String>) -> Self {
        Self::VecString(x)
    }
}

impl From<Vec<DateTime<Local>>> for Value {
    fn from(x: Vec<DateTime<Local>>) -> Self {
        Self::VecDateTime(x)
    }
}
