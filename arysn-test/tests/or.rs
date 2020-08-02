use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn or() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    // WEHER active = TRUE OR age = 21
    let users = User::select()
        .active()
        .eq(true)
        .or()
        .age()
        .eq(21)
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 3);

    // WHERE active = TRUE AND (age = 21 OR age = 22)
    let users = User::select()
        .active()
        .eq(true)
        // r#where って名前どうなの？ でも()付けるだけなんだよね
        .r#where(|b| b.age().eq(21).or().age().eq(22))
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);

    Ok(())
}
