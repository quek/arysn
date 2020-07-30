use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn many_to_many() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    let users = User::select()
        .contributions(|contribution| {
            contribution
                .preload()
                .project(|project| project.id().is_not_null().preload())
        })
        .order()
        .id()
        .asc()
        .load(conn)
        .await?;
    assert_eq!(users.len(), 2);
    let user = &users[0];
    assert_eq!(user.contributions.len(), 3);

    Ok(())
}
