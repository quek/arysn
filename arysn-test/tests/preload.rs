use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::enums::RoleType;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn preload() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let users = User::select()
        .roles(|roles| roles.role_type().eq(RoleType::Admin))
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    let user = &users[0];
    assert!(user.roles.is_empty());

    let users = User::select()
        .roles(|roles| roles.role_type().eq(RoleType::Admin).preload())
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    let user = &users[0];
    let roles = &user.roles;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].role_type, RoleType::Admin);

    let users = User::select()
        .roles(|roles| roles.preload().role_type().eq(RoleType::Admin))
        .order()
        .id()
        .asc()
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 3);
    let user = &users[0];
    let roles = &user.roles;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].role_type, RoleType::Admin);
    let user = &users[1];
    let roles = &user.roles;
    assert_eq!(roles.len(), 0);
    let user = &users[2];
    let roles = &user.roles;
    assert_eq!(roles.len(), 0);

    Ok(())
}
