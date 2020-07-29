use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn or() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    // WEHER active = TRUE OR age = 21
    let users = User::select()
        .active()
        .eq(true)
        .or()
        .age()
        .eq(21)
        .load(conn)
        .await?;
    assert_eq!(users.len(), 3);

    // // WHERE active = TRUE AND (age = 10 OR age = 20 OR age = 30)
    // let users = User::select()
    //     .active()
    //     .eq(true)
    //     .r#where(|b| b.age().eq(10).or().age().eq(2).or().age().eq(30))
    //     .load(conn).await?;
    // assert_eq!(users.len(), 1);

    Ok(())
}
