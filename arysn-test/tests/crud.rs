use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::project::Project;
use arysn_test::generated::user::{User, UserNew};
use common::init;

mod common;

#[tokio::test]
async fn crud() -> Result<()> {
    init();

    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    let users = User::select()
        .id()
        .eq(1)
        .name()
        .eq("ユーザ1".to_string())
        .load(conn)
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
    user.update(conn).await?;

    let user = User::select().id().eq(1).first(conn).await?;
    assert_eq!(user.age, age);

    let user = UserNew {
        id: None,
        name: "こねら".to_string(),
        title: Some("さば".to_string()),
        age: 3,
        active: true,
        created_at: None,
    };
    let user = user.insert(conn).await?;
    assert_eq!(user.name, "こねら".to_string());
    assert_eq!(user.title, Some("さば".to_string()));
    assert_eq!(user.age, 3);
    assert_eq!(user.active, true);
    user.delete(conn).await?;
    let user = User::select().id().eq(user.id).first(conn).await;
    log::debug!("{:?}", &user);
    assert_eq!(user.is_err(), true);

    Ok(())
}

#[tokio::test]
async fn limit_offset() -> Result<()> {
    init();

    let conn = &connect().await?;

    let users = User::select().load(conn).await?;
    assert_eq!(users.len(), 3);

    let users = User::select().limit(1).offset(1).load(conn).await?;
    assert_eq!(users.len(), 1);

    Ok(())
}

#[tokio::test]
async fn literal_condition() -> Result<()> {
    init();

    let conn = &connect().await?;

    let projects = Project::select()
        .literal_condition("create_user_id=update_user_id")
        .load(conn)
        .await?;
    assert_eq!(projects.len(), 1);

    Ok(())
}

#[tokio::test]
async fn operators() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    let users = User::select()
        .id()
        .gt(1)
        .id()
        .lt(3)
        .order()
        .id()
        .asc()
        .load(conn)
        .await?;
    assert_eq!(users[0].id, 2);

    Ok(())
}
