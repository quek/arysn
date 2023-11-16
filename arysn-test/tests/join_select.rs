use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::project::Project;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn join_as_has_many() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    let projects = Project::select()
        .id()
        .gte(0)
        .group_by_literal("create_user_id");
    let users = User::select()
        .age()
        .gt(9)
        .join_select(
            "create_user_id, MAX(name) AS last_name",
            projects,
            "pj",
            "pj.create_user_id = users.id",
        )
        .literal_condition_with_args("last_name like $1", vec!["ねこみみ%"])
        .order()
        .id()
        .asc()
        .load(conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 1);

    Ok(())
}
