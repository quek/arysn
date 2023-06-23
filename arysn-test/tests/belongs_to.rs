use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::{role::Role, user::User};
use common::init;

mod common;

#[tokio::test]
async fn belongs_to() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;
    let roles = Role::select()
        .user(|user| user.id().eq(1))
        .load(conn)
        .await?;
    assert_eq!(roles.len(), 2);

    let roles = Role::select()
        .user(|user| user.preload())
        .load(conn)
        .await?;
    assert_eq!(roles[0].user.is_some(), true);

    let user = User::select()
        .id()
        .eq(1)
        .create_projects(|projcects| {
            projcects
                .create_user(|user| user.preload())
                .update_user(|user| user.id().eq(2).preload())
                .check_user(|user| user.preload())
                .preload()
        })
        .first(conn)
        .await?;
    assert_eq!(user.create_projects[0].create_user().name, "ユーザ1");
    assert_eq!(user.create_projects[0].update_user().name, "ユーザ2");
    assert_eq!(user.create_projects[0].check_user, None);

    Ok(())
}
