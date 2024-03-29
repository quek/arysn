use crate::filter::{Column, Filter};
use crate::value::ToSqlValue;
use std::marker::PhantomData;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum RelationType {
    #[default]
    TopLevel,
    HasMany(&'static str),
    HasOne(&'static str),
    BelongsTo(&'static str),
}

pub trait BuilderAccessor {
    fn table_name(&self) -> &String;
    fn table_name_as(&self) -> &Option<String>;
    fn table_name_as_or(&self) -> &String {
        self.table_name_as().as_ref().unwrap_or(self.table_name())
    }
    fn filters(&self) -> &Vec<Filter>;
    fn filters_mut(&mut self) -> &mut Vec<Filter>;
    fn join_selects(&self) -> &Vec<JoinSelect>;
    fn outer_join(&self) -> bool;
    fn preload(&self) -> bool;
    fn relation_type(&self) -> &RelationType;
}

pub struct FilterBuilder<B, V> {
    pub column_name: &'static str,
    pub builder: B,
    pub value_type: PhantomData<V>,
}

impl<B, V> FilterBuilder<B, V>
where
    B: BuilderAccessor + Clone,
    V: ToSqlValue + 'static,
{
    pub fn eq(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "=",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn gt(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: ">",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn lt(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "<",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn gte(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: ">=",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn lte(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "<=",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn not_eq(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "<>",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn is_null(&self) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![],
            operator: "IS NULL",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn is_not_null(&self) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![],
            operator: "IS NOT NULL",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn between(&self, from: V, to: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(from), Box::new(to)],
            operator: "BETWEEN",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn r#in(&self, values: Vec<V>) -> B {
        let mut builder = self.builder.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vs,
            operator: "IN",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn not_in(&self, values: Vec<V>) -> B {
        let mut builder = self.builder.clone();
        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
        for v in values {
            vs.push(Box::new(v));
        }
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vs,
            operator: "NOT IN",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
    pub fn like(&self, value: V) -> B {
        let mut builder = self.builder.clone();
        let filter = Filter::Column(Column {
            table: BuilderAccessor::table_name_as(&builder)
                .as_ref()
                .unwrap_or(BuilderAccessor::table_name(&builder))
                .to_string(),
            name: self.column_name.to_string(),
            values: vec![Box::new(value)],
            operator: "LIKE",
            preload: BuilderAccessor::preload(&builder),
        });
        BuilderAccessor::filters_mut(&mut builder).push(filter);
        builder
    }
}

#[derive(Clone, Debug)]
pub struct JoinSelect {
    pub select: &'static str,
    pub builder: Box<dyn crate::prelude::BuilderTrait>,
    pub as_name: &'static str,
    pub on: &'static str,
}
