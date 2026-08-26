use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn for_update() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    // FOR UPDATE は DISTINCT と併用できないので DISTINCT は付かない
    assert_eq!(
        User::select().id().eq(1).for_update().select_sql(),
        "SELECT users.id, users.name, users.title, users.age, users.active, \
         users.start_time, users.created_at FROM users WHERE users.id = $1 FOR UPDATE"
    );

    let user = User::select().id().eq(1).for_update().first(&conn).await?;
    assert_eq!(user.id, 1);

    let users = User::select().for_update().load(&conn).await?;
    assert_eq!(users.len(), 3);

    Ok(())
}

#[tokio::test]
async fn for_update_locks_row() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let user = User::select().id().eq(1).for_update().first(&conn).await?;
    assert_eq!(user.id, 1);

    // 別コネクションからは同じ行をロックできない
    let mut other = connect().await?;
    let other = other.transaction().await?;
    let result = other
        .query("SELECT id FROM users WHERE id = 1 FOR UPDATE NOWAIT", &[])
        .await;
    assert!(result.is_err());

    Ok(())
}
