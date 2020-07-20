use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::{User, UserNew};
use common::init;

mod common;

#[tokio::test]
async fn crud() -> Result<()> {
    init();

    let client = connect().await?;

    let users = User::select()
        .id()
        .eq(1)
        .name()
        .eq("ユーザ1".to_string())
        .load(&client)
        .await?;
    assert_eq!(users.len(), 1);
    let user = &users[0];
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "ユーザ1");
    assert_eq!(user.title, Some("旅人".to_string()));
    assert_eq!(user.active, true);

    let mut user = user.clone();
    let age = user.age + 100;
    user.age = age;
    user.update(&client).await?;

    let user = User::select().id().eq(1).first(&client).await?;
    assert_eq!(user.age, age);

    let user = UserNew {
        id: None,
        name: "こねら".to_string(),
        title: Some("さば".to_string()),
        age: 3,
        active: true,
        created_at: None,
    };
    let user = user.insert(&client).await?;
    assert_eq!(user.name, "こねら".to_string());
    assert_eq!(user.title, Some("さば".to_string()));
    assert_eq!(user.age, 3);
    assert_eq!(user.active, true);
    user.delete(&client).await?;
    let user = User::select().id().eq(user.id).first(&client).await;
    log::debug!("{:?}", &user);
    assert_eq!(user.is_err(), true);

    Ok(())
}

#[tokio::test]
async fn limit_offset() -> Result<()> {
    init();

    let cnn = &connect().await?;

    let users = User::select().load(cnn).await?;
    assert_eq!(users.len(), 3);

    let users = User::select().limit(1).offset(1).load(cnn).await?;
    assert_eq!(users.len(), 1);

    Ok(())
}
