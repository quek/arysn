use tokio_postgres::types::ToSql;
use dyn_clone::DynClone;

pub trait ToSqlValue: DynClone + ToSql + Sync + Send {
    fn as_to_sql(&self) -> Option<&(dyn ToSql + Sync)>;
}

impl<T> ToSqlValue for T where T: ToSql + Sync + Clone + Send {
    fn as_to_sql(&self) -> Option<&(dyn ToSql + Sync)> {
        Some(self)
    }
}

dyn_clone::clone_trait_object!(ToSqlValue);
