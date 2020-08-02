use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::role::RoleType;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn has_many() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let users = User::select()
        .active()
        .eq(true)
        .roles(|roles| roles.role_type().eq(RoleType::Admin))
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 1);
    assert_eq!(users[0].roles.is_empty(), true);

    let users = User::select()
        .roles(|roles| {
            roles
                .preload()
                .screens(|screens| screens.id().eq(1).preload())
        })
        .load(&conn)
        .await?;
    let screen = &users[0].roles[0].screens[0];
    assert_eq!(screen.id, 1);

    Ok(())
}
