use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::{enums::RoleType, user::User};
use common::init;

mod common;

#[tokio::test]
async fn r#as() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let users = User::select()
        .roles(|roles| roles.r#as("rl".to_string()).role_type().eq(RoleType::Admin))
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    Ok(())
}
