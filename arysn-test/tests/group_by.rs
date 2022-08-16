use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn group_by() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let sql = User::select()
        .create_projects(|project| project.id().is_not_null())
        .group_by_literal("users.id");
    let users = sql
        .order()
        .by_string_literal_asc("MIN(create_projects.id)")
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, 1);

    let users = sql
        .order()
        .by_string_literal_asc("MAX(create_projects.id)")
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, 2);

    Ok(())
}
