use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::gis_thing::GisThing;
use common::init;

mod common;

#[tokio::test]
async fn gis() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    let things = GisThing::select().load(&conn).await?;
    println!(
        "=============================================================================\n{:#?}",
        things
    );

    Ok(())
}
