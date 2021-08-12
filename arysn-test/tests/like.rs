#[cfg(feature = "with-tokio-0_2")]
extern crate tokio_0_2 as tokio;
#[cfg(feature = "with-tokio-1_x")]
extern crate tokio_1_x as tokio;

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
