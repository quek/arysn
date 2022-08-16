use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::project::Project;
use common::init;

mod common;

#[tokio::test]
async fn like() -> Result<()> {
    init();
    let conn = connect().await?;

    let projects = Project::select()
        .name()
        .like("ねこ%".to_string())
        .load(&conn)
        .await?;
    assert_eq!(projects.len(), 3);

    let projects = Project::select()
        .name()
        .like(format!("{}%", arysn::escape_like("ねこ%")))
        .load(&conn)
        .await?;
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "ねこ%手企画(1)");

    Ok(())
}
