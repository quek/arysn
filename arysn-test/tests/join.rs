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

#[tokio::test]
async fn join_as_has_many() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    let users = User::select()
        .order()
        .id()
        .asc()
        .create_projects(|x| x.id().r#in(vec![1, 2]).preload())
        .update_projects(|x| x.id().r#in(vec![1, 3]).preload())
        .load(conn)
        .await?;
    assert_eq!(users.len(), 2);
    let user = &users[0];
    assert_eq!(user.id, 1);
    let create_projects = &user.create_projects;
    assert_eq!(create_projects.len(), 1);
    let update_projects = &user.update_projects;
    assert_eq!(update_projects.len(), 1);
    let user = &users[1];
    assert_eq!(user.id, 2);
    let create_projects = &user.create_projects;
    assert_eq!(create_projects.len(), 1);
    let update_projects = &user.update_projects;
    assert_eq!(update_projects.len(), 1);

    Ok(())
}
