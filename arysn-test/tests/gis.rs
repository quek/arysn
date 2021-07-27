mod common;

#[cfg(feature = "gis")]
mod gis {
    #[cfg(feature = "with-tokio-0_2")]
    extern crate tokio_0_2 as tokio;
    #[cfg(feature = "with-tokio-1_x")]
    extern crate tokio_1_x as tokio;

    use super::common::init;
    use anyhow::Result;
    use arysn::prelude::*;
    use arysn_test::generated::gis_thing::{GisThing, GisThingNew};

    #[tokio::test]
    async fn gis() -> Result<()> {
        init();
        let mut conn = connect().await?;
        let conn = &conn.transaction().await?;

        let thing = GisThing::select()
            .order()
            .id()
            .asc()
            .limit(1)
            .first(&conn)
            .await?;
        assert_eq!(thing.latlng.x, 139.71019683);
        assert_eq!(thing.latlng.y, 35.66432955);

        GisThingNew {
            id: None,
            latlng: arysn::Point::new(139.7101234, 35.6641234, None),
        }
        .insert(&conn)
        .await?;
        let thing = GisThing::select()
            .order()
            .id()
            .desc()
            .limit(1)
            .first(&conn)
            .await?;
        assert_eq!(thing.latlng.x, 139.7101234);
        assert_eq!(thing.latlng.y, 35.6641234);

        Ok(())
    }
}
