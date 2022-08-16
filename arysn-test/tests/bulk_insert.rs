use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::{User, UserNew};
use common::init;

mod common;

#[tokio::test]
async fn bulk_insert() -> Result<()> {
    init();

    let mut conn = connect().await?;
    let conn = &conn.transaction().await?;

    let users = vec![
        UserNew {
            id: None,
            name: "こねら".to_string(),
            title: Some("さば".to_string()),
            age: 3,
            active: true,
            created_at: None,
        },
        UserNew {
            id: None,
            name: "ぎょぴ".to_string(),
            title: Some("いわし".to_string()),
            age: 12,
            active: true,
            created_at: None,
        },
    ];
    let users = User::insert(&users, conn).await?;
    assert_eq!(users.len(), 2);
    assert!(users.iter().all(|x| x.id > 0));
    assert_eq!(users[0].name, "こねら");
    assert_eq!(users[1].name, "ぎょぴ");

    Ok(())
}
