use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::project::Project;
use common::init;

mod common;

#[tokio::test]
async fn join_as() -> Result<()> {
    init();
    let conn = &connect().await?;

    let projects = Project::select()
        .create_user(|x| x.preload().id().eq(2))
        .load(conn)
        .await?;
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].create_user.as_ref().unwrap().id, 2);

    Ok(())
}
