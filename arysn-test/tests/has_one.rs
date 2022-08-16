use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use chrono::NaiveDate;
use common::init;

mod common;

#[tokio::test]
async fn has_one() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let users = User::select()
        .profile(|profile| profile.preload())
        .order()
        .id()
        .asc()
        .load(&conn)
        .await?;
    // preload だけなら profile のないのもとってくる
    assert_eq!(users.len(), 3);
    let user = &users[0];
    let profile = user.profile.as_ref().unwrap();
    assert_eq!(profile.birth_date, NaiveDate::from_ymd(1999, 12, 31));
    let user = &users[1];
    let profile = user.profile.as_ref().unwrap();
    assert_eq!(profile.birth_date, NaiveDate::from_ymd(2000, 1, 1));

    let users = User::select()
        .profile(|profile| profile.id().is_not_null().preload())
        .order()
        .id()
        .asc()
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 2);

    let users = User::select()
        .profile(|profile| {
            profile
                .birth_date()
                .lt(NaiveDate::from_ymd(2000, 1, 1))
                .preload()
        })
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(
        users[0].profile.as_ref().unwrap().birth_date,
        NaiveDate::from_ymd(1999, 12, 31)
    );

    Ok(())
}
