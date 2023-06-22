use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::project::Project;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn join_as_belongs_to() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    let projects = Project::select()
        .create_user(|x| x.id().eq(2).preload())
        .update_user(|x| x.preload())
        .load(conn)
        .await?;
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].create_user.as_ref().unwrap().id, 2);
    assert_eq!(projects[0].update_user.as_ref().unwrap().id, 1);

    Ok(())
}

// TODO #[tokio::test]
// TODO async fn join_as_has_many() -> Result<()> {
// TODO     init();
// TODO     let mut conn = connect().await?;
// TODO     let conn = &conn.transaction().await?;
// TODO
// TODO     let users = User::select()
// TODO         .order()
// TODO         .id()
// TODO         .asc()
// TODO         .create_projects(|x| x.id().r#in(vec![1, 2]).preload())
// TODO         .update_projects(|x| x.id().r#in(vec![1, 3]).preload())
// TODO         .load(conn)
// TODO         .await?;
// TODO     assert_eq!(users.len(), 2);
// TODO     let user = &users[0];
// TODO     assert_eq!(user.id, 1);
// TODO     let create_projects = &user.create_projects;
// TODO     assert_eq!(create_projects.len(), 1);
// TODO     let update_projects = &user.update_projects;
// TODO     assert_eq!(update_projects.len(), 1);
// TODO     let user = &users[1];
// TODO     assert_eq!(user.id, 2);
// TODO     let create_projects = &user.create_projects;
// TODO     assert_eq!(create_projects.len(), 1);
// TODO     let update_projects = &user.update_projects;
// TODO     assert_eq!(update_projects.len(), 1);
// TODO
// TODO     Ok(())
// TODO }
