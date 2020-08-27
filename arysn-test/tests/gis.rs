use anyhow::Result;
use arysn::prelude::*;
// use arysn_test::generated::gis_thing::GisThing;
use common::init;

mod common;

#[tokio::test]
async fn simple() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    conn.query("SELECT * FROM gis_things", &[]).await?;

    Ok(())
}
