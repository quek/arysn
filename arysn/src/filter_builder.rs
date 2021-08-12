use crate::filter::Filter;
use crate::value::ToSqlValue;
use std::marker::PhantomData;

pub trait BuilderAccessor: Clone {
    fn from(x: &Self) -> &String;
    fn table_name_as(x: &Self) -> &Option<String>;
    fn filters(x: &mut Self) -> &mut Vec<Filter>;
    fn preload(x: &Self) -> bool;
}

pub struct FilterBuilder<B, V> {
    pub column_name: &'static str,
    pub builder: B,
    pub value_type: PhantomData<V>,
}

impl<B, V> FilterBuilder<B, V>
where
    B: BuilderAccessor,
    V: ToSqlValue + 'static,
{
    pub fn eq(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "=",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn gt(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: ">",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn lt(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "<",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn gte(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: ">=",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn lte(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "<=",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn not_eq(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "<>",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn is_null(&self) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![],
            operator: "IS NULL",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn is_not_null(&self) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![],
            operator: "IS NOT NULL",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn between(&self, from: V, to: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(from), Box::new(to)],
            operator: "BETWEEN",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn r#in(&self, values: Vec<V>) -> B {
        let mut builder = self.builder.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vs,
            operator: "IN",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn not_in(&self, values: Vec<V>) -> B {
        let mut builder = self.builder.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vs,
            operator: "NOT IN",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
    pub fn like(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::from(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "LIKE",
            preload: BuilderAccessor::preload(&builder),
        };
        BuilderAccessor::filters(&mut builder).push(filter);
        builder
    }
}
