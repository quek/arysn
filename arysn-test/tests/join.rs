use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::project::Project;
use common::init;

mod common;

#[tokio::test]
async fn join() -> Result<()> {
    init();
    let conn = &connect().await?;

    let project = Project::select()
        .create_user(|x| x.preload())
        .id()
        .eq(1)
        .first(conn)
        .await?;

    Ok(())
}
