use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn order_by_string_literal() -> Result<()> {
    init();
    let conn = connect().await?;

    let users = User::select()
        .profile(|x| x.id().is_not_null())
        .order()
        .by_string_literal_asc("profiles.birth_date")
        .load(&conn)
        .await?;
    assert_eq!(users[0].id, 1);
    assert_eq!(users[1].id, 2);

    let users = User::select()
        .profile(|x| x.id().is_not_null())
        .order()
        .by_string_literal_desc("profiles.birth_date")
        .load(&conn)
        .await?;
    assert_eq!(users[0].id, 2);
    assert_eq!(users[1].id, 1);

    Ok(())
}
