use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::enums::RoleType;
use arysn_test::generated::screen::Screen;
use arysn_test::generated::user::User;
use common::init;

mod common;

#[tokio::test]
async fn has_many() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let users = User::select()
        .active()
        .eq(true)
        .roles(|roles| roles.role_type().eq(RoleType::Admin))
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 1);
    assert_eq!(users[0].roles.is_empty(), true);

    let screen = Screen::select()
        .name()
        .eq("ねこ".to_string())
        .first(&conn)
        .await?;
    let users = User::select()
        .roles(|roles| {
            roles
                .preload()
                .screens(|screens| screens.id().eq(screen.id).preload())
        })
        .load(&conn)
        .await?;
    let screen = &users[0].roles[0].screens[0];
    assert_eq!(screen.id, screen.id);

    Ok(())
}

#[tokio::test]
async fn has_many_as_preload() -> Result<()> {
    init();
    let conn = connect().await?;

    // INNER JOIN ... ON contributions.project_id = create_projects.id
    // ではなく
    // INNER JOIN ... ON contributions.project_id = projects.id
    // になること
    let users = User::select()
        .create_projects(|x| x.contributions(|x| x.id().eq(1).preload()).preload())
        .load(&conn)
        .await?;
    assert_eq!(users[0].create_projects[0].contributions[0].id, 1);

    Ok(())
}
