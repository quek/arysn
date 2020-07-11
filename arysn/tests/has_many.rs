use anyhow::Result;
use arysn::prelude::*;
use common::{init, User};

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
    assert_eq!(1, users.len());
    assert_eq!(Some(1), users[0].id);
    Ok(())
}
