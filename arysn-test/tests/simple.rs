use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::simple::Simple;
use common::init;

mod common;

// warning: unused variable: `join_parts`
// warning: variable does not need to be mutable
// などのチェク用
#[tokio::test]
async fn simple() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    let simples = Simple::select().load(conn).await?;
    assert_eq!(simples.len(), 0);

    let err = Simple::select().first(conn).await;
    match err {
        Err(arysn::Error::NotFound) => assert!(true),
        _ => assert!(false),
    }

    Ok(())
}
