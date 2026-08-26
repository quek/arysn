use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::enums::RoleType;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn max_min() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let max: Option<i32> = User::select().age().max(&conn).await?;
    assert_eq!(max, Some(22));

    let min: Option<i32> = User::select().age().min(&conn).await?;
    assert_eq!(min, Some(20));

    let max: Option<String> = User::select().name().max(&conn).await?;
    assert_eq!(max, Some("ユーザ3".to_string()));

    Ok(())
}

#[tokio::test]
async fn max_min_with_condition() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let max: Option<i32> = User::select().active().eq(false).age().max(&conn).await?;
    assert_eq!(max, Some(21));

    let min: Option<i32> = User::select().age().gte(21).age().min(&conn).await?;
    assert_eq!(min, Some(21));

    // 該当行がなければ NULL
    let max: Option<i32> = User::select().age().gt(100).age().max(&conn).await?;
    assert_eq!(max, None);

    Ok(())
}

#[tokio::test]
async fn max_min_nullable_column() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    // NULL は無視される
    let max: Option<chrono::NaiveTime> = User::select().start_time().max(&conn).await?;
    assert_eq!(max, Some(chrono::NaiveTime::from_hms_opt(7, 8, 9).unwrap()));

    Ok(())
}

#[tokio::test]
async fn max_min_with_join() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let max: Option<i64> = User::select()
        .roles(|role| role.role_type().eq(RoleType::Admin))
        .id()
        .max(&conn)
        .await?;
    assert_eq!(max, Some(1));

    let max: Option<i64> = User::select()
        .roles(|role| role.role_type().eq(RoleType::User))
        .id()
        .max(&conn)
        .await?;
    assert_eq!(max, Some(2));

    Ok(())
}
