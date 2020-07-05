#[derive(Clone, Debug)]
pub enum Value {
    I64(i64),
    String(String),
}

impl Value {
    pub fn to_sql_value(&self) -> String {
        match self {
            Self::I64(x) => x.to_string(),
            Self::String(x) => format!("'{}'", x.replace("'", "''")),
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
