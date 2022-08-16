use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::role::Role;
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

    Ok(())
}
