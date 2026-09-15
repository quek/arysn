use anyhow::Result;
use arysn::prelude::*;
use arysn_test::generated::enums::RoleType;
use arysn_test::generated::project::Project;
use arysn_test::generated::role::Role;
use arysn_test::generated::screen::Screen;
use arysn_test::generated::user::User;
use chrono::NaiveTime;
use common::init;

mod common;

#[tokio::test]
async fn any() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    // 値の数に関わらず配列 1 つのパラメータになる
    let builder = User::select().id().any(vec![1, 3]);
    assert_eq!(
        builder.select_sql(),
        "SELECT DISTINCT users.id, users.name, users.title, users.age, users.active, \
         users.start_time, users.created_at FROM users WHERE users.id = ANY($1)"
    );
    assert_eq!(builder.select_params().len(), 1);

    let users = builder.order().id().asc().load(&conn).await?;
    assert_eq!(users.iter().map(|x| x.id).collect::<Vec<_>>(), vec![1, 3]);

    let user = User::select().id().any(vec![2]).first(&conn).await?;
    assert_eq!(user.id, 2);

    let users = User::select()
        .name()
        .any(vec!["ユーザ1".to_string(), "ユーザ3".to_string()])
        .order()
        .id()
        .asc()
        .load(&conn)
        .await?;
    assert_eq!(users.iter().map(|x| x.id).collect::<Vec<_>>(), vec![1, 3]);

    Ok(())
}

#[tokio::test]
async fn any_empty() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    // 空配列なら何にもマッチしない
    let users = User::select().id().any(vec![]).load(&conn).await?;
    assert!(users.is_empty());

    let count = User::select().id().any(vec![]).count(&conn).await?;
    assert_eq!(count, 0);

    Ok(())
}

#[tokio::test]
async fn any_many_values() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    // r#in は値ごとにパラメータを使うので tokio-postgres の上限 (i16::MAX) を超えるとエラーになるが
    // any はパラメータ 1 つなので値が多くても大丈夫
    let ids: Vec<i64> = (1..=40000).collect();
    let count = User::select().id().any(ids).count(&conn).await?;
    assert_eq!(count, 3);

    Ok(())
}

#[tokio::test]
async fn any_bind_index() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let builder = User::select()
        .active()
        .eq(true)
        .id()
        .any(vec![1, 2, 3])
        .age()
        .gt(20);
    let sql = builder.select_sql();
    assert!(sql.ends_with(
        "WHERE users.active = $1 AND users.id = ANY($2) AND users.age > $3"
    ));
    assert_eq!(builder.select_params().len(), 3);
    let users = builder.load(&conn).await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 3);

    let builder = User::select()
        .id()
        .r#in(vec![1, 2, 3])
        .age()
        .any(vec![20, 22])
        .name()
        .not_eq("ユーザ1".to_string());
    let sql = builder.select_sql();
    assert!(sql.ends_with(
        "WHERE users.id IN ($1, $2, $3) AND users.age = ANY($4) AND users.name <> $5"
    ));
    assert_eq!(builder.select_params().len(), 5);
    let users = builder.load(&conn).await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 3);

    let users = User::select()
        .r#where(|user| user.id().any(vec![1]).or().age().any(vec![22]))
        .order()
        .id()
        .asc()
        .load(&conn)
        .await?;
    assert_eq!(users.iter().map(|x| x.id).collect::<Vec<_>>(), vec![1, 3]);

    Ok(())
}

#[tokio::test]
async fn any_types() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    // enum
    let roles = Role::select()
        .role_type()
        .any(vec![RoleType::Admin])
        .load(&conn)
        .await?;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].role_type, RoleType::Admin);

    let roles = Role::select()
        .role_type()
        .any(vec![RoleType::Admin, RoleType::User])
        .load(&conn)
        .await?;
    assert_eq!(roles.len(), 3);

    // nullable なカラム
    let users = User::select()
        .title()
        .any(vec!["旅人".to_string()])
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 1);

    // TIME
    let users = User::select()
        .start_time()
        .any(vec![NaiveTime::from_hms_opt(7, 8, 9).unwrap()])
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 1);

    // UUID
    let screens = Screen::select()
        .name()
        .r#in(vec!["ねこ".to_string(), "のり".to_string()])
        .load(&conn)
        .await?;
    assert_eq!(screens.len(), 2);
    let screens = Screen::select()
        .id()
        .any(screens.iter().map(|x| x.id).collect())
        .order()
        .name()
        .asc()
        .load(&conn)
        .await?;
    assert_eq!(
        screens.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(),
        vec!["ねこ", "のり"]
    );

    Ok(())
}

#[tokio::test]
async fn any_join() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    let users = User::select()
        .roles(|role| role.role_type().any(vec![RoleType::Admin]))
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 1);
    assert!(users[0].roles.is_empty());

    let count = User::select()
        .roles(|role| role.role_type().any(vec![RoleType::User]))
        .count(&conn)
        .await?;
    assert_eq!(count, 2);

    let max: Option<i64> = User::select()
        .roles(|role| role.role_type().any(vec![RoleType::User]))
        .id()
        .max(&conn)
        .await?;
    assert_eq!(max, Some(2));

    let projects = Project::select()
        .create_user(|user| user.id().any(vec![2]))
        .load(&conn)
        .await?;
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, 2);

    Ok(())
}

#[tokio::test]
async fn any_preload() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    // preload より前の条件は join に使われる
    let users = User::select()
        .roles(|role| role.role_type().any(vec![RoleType::Admin]).preload())
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].roles.len(), 1);
    assert_eq!(users[0].roles[0].role_type, RoleType::Admin);

    // preload より後の条件は preload のクエリにだけ使われる
    let users = User::select()
        .roles(|role| role.preload().role_type().any(vec![RoleType::User]))
        .order()
        .id()
        .asc()
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 3);
    assert_eq!(users[0].roles.len(), 1);
    assert_eq!(users[0].roles[0].role_type, RoleType::User);
    assert_eq!(users[1].roles.len(), 1);
    assert_eq!(users[2].roles.len(), 0);

    let projects = Project::select()
        .id()
        .any(vec![1, 2])
        .create_user(|user| user.preload().id().any(vec![1]))
        .order()
        .id()
        .asc()
        .load(&conn)
        .await?;
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].create_user.as_ref().map(|x| x.id), Some(1));
    assert!(projects[1].create_user.is_none());

    Ok(())
}

#[tokio::test]
async fn any_join_select() -> Result<()> {
    init();
    let mut conn = connect().await?;
    let conn = conn.transaction().await?;

    // join_select のサブクエリ内のバインドとの番号の整合
    let projects = Project::select()
        .id()
        .any(vec![1, 2, 3])
        .group_by_literal("create_user_id");
    let users = User::select()
        .join_select(
            "create_user_id, COUNT(*) AS project_count",
            projects,
            "pj",
            "pj.create_user_id = users.id",
        )
        .id()
        .any(vec![1, 2])
        .literal_condition_with_args("pj.project_count = $1", vec![2i64])
        .load(&conn)
        .await?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 1);

    Ok(())
}
