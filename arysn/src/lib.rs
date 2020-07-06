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
    use arysn_macro::define_ar;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    define_ar!(User {
        table_name: users,
        has_many: roles
    });

    define_ar!(Roles {
        table_name: roles,
        belogns_to: user
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
        assert_eq!(Some(1), user.id);
        assert_eq!("ユーザ1", user.name);
        assert_eq!(Some("旅人".to_string()), user.title);
        assert_eq!(true, user.active);
        log::debug!("{}", user.created_at);

        let mut user = user.clone();
        let age = user.age + 100;
        user.age = age;
        user.update(&client).await?;

        let user = User::select().id().eq(1).first(&client).await?;
        assert_eq!(age, user.age);

        let created_at = chrono::Local::now();
        let user = User {
            id: None,
            name: "こねら".to_string(),
            title: Some("さば".to_string()),
            age: 3,
            active: true,
            created_at,
        };
        let user = user.insert(&client).await?;
        assert_eq!(true, user.id.is_some());
        assert_eq!("こねら".to_string(), user.name);
        assert_eq!(Some("さば".to_string()), user.title);
        assert_eq!(3, user.age);
        assert_eq!(true, user.active);
        // nano seconds が postgres の方にない
        assert_eq!(
            created_at.format("'%Y-%m-%d %H:%M:%S%.6f %:z'").to_string(),
            user.created_at
                .format("'%Y-%m-%d %H:%M:%S%.6f %:z'")
                .to_string()
        );
        user.delete(&client).await?;
        let user = User::select()
            .id()
            .eq(user.id.unwrap())
            .first(&client)
            .await;
        log::debug!("{:?}", &user);
        assert_eq!(true, user.is_err());

        Ok(())
    }

    //User::select().join().roles().merge(Roles::select().active().eq(true))
}
