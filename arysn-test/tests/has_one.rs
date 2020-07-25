use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::user::User;
use chrono::NaiveDate;
use common::init;

mod common;

#[tokio::test]
async fn has_one() -> Result<()> {
    init();
    let conn = &connect().await?;

    let users = User::select()
        .profile(|profile| profile.preload())
        .load(conn)
        .await?;
    assert_eq!(users.len(), 2);
    let user = &users[0];
    let profile = user.profile.unwrap();
    assert_eq!(profile.birth_date, NaiveDate::from_ymd(1999, 12, 31));
    let user = &users[1];
    let profile = user.profile.unwrap();
    assert_eq!(profile.birth_date, NaiveDate::from_ymd(2000, 1, 1));

    Ok(())
}
