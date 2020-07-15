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

    let users = User::select()
        .roles(|roles| {
            roles
                .preload()
                .screens(|screens| screens.id().eq(1).preload())
        })
        .load(&client)
        .await?;
    let screen = &users[0].roles.as_ref().unwrap()[0]
        .screens
        .as_ref()
        .unwrap()[0];
    assert_eq!(screen.id, 1);

    Ok(())
}
