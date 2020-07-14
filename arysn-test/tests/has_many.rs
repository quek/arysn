use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn has_many() -> Result<()> {
    init();
    let client = connect().await?;

    let users = User::select()
        .active()
        .eq(true)
        .roles(|roles| roles.name().eq("管理".to_string()))
        .load(&client)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 1);
    assert_eq!(users[0].roles.is_none(), true);

    let users = User::select()
        .active()
        .eq(true)
        .roles(|roles| roles.name().eq("管理".to_string()).preload())
        .load(&client)
        .await?;
    assert_eq!(users[0].roles.is_some(), true);

    Ok(())
}
