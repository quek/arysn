use anyhow::Result;
use arysn::prelude::*;
use common::{init, Role};

mod common;

#[tokio::test]
async fn belongs_to() -> Result<()> {
    init();
    let connection = connect().await?;
    let roles = Role::select()
        .user(|user| user.id().eq(1))
        .load(&connection)
        .await?;
    assert_eq!(roles.len(), 2);
    Ok(())
}
