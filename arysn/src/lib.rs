mod bulider;
mod db;
mod filter;
mod value;

pub mod prelude {
    pub use super::bulider::BuilderTrait;
    pub use super::db::connect;
    pub use super::filter::Filter;
    pub use super::value::Value;
}

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use anyhow::Result;
    use arysn_macro::defar;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    defar!(User {
        table_name: users,
        foo: bar
    });

    #[tokio::test]
    async fn it_works() -> Result<()> {
        init();

        let client = connect().await?;

        let users = User::select()
            .id()
            .eq(1)
            .name()
            .eq("ユーザ1".to_string())
            .load(&client)
            .await?;
        assert_eq!(1, users.len());
        let user = &users[0];
        assert_eq!(1, user.id);
        assert_eq!("ユーザ1", user.name);
        assert_eq!(Some("旅人".to_string()), user.title);

        Ok(())
    }
}
