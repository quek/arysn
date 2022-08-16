use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn outer_join() -> Result<()> {
    init();
    let conn = connect().await?;

    let users = User::select()
        .roles(|x| x.outer_join().id().is_null())
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 3);

    Ok(())
}
