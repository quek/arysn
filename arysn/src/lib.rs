#[cfg(feature = "with-tokio-0_2")]
extern crate deadpool_0_5 as deadpool;
#[cfg(feature = "with-tokio-0_2")]
extern crate deadpool_postgres_0_5 as deadpool_postgres;
#[cfg(feature = "with-tokio-0_2")]
extern crate tokio_0_2 as tokio;
#[cfg(feature = "with-tokio-0_2")]
extern crate tokio_postgres_0_5 as tokio_postgres;

#[cfg(feature = "with-tokio-1_x")]
extern crate deadpool_0_8 as deadpool;
#[cfg(feature = "with-tokio-1_x")]
extern crate deadpool_postgres_0_9 as deadpool_postgres;
#[cfg(feature = "with-tokio-1_x")]
extern crate tokio_1_x as tokio;
#[cfg(feature = "with-tokio-1_x")]
extern crate tokio_postgres_0_7 as tokio_postgres;

#[cfg(feature = "gis-tokio-0_2")]
extern crate bytes_0_5 as bytes;
#[cfg(feature = "gis-tokio-0_2")]
extern crate postgis_0_7 as postgis;
#[cfg(feature = "gis-tokio-0_2")]
extern crate postgres_0_17 as postgres;

#[cfg(feature = "gis")]
extern crate bytes_1_x as bytes;
#[cfg(feature = "gis")]
extern crate postgis_0_8 as postgis;
#[cfg(feature = "gis")]
extern crate postgres_0_19 as postgres;

mod builder;
mod db;
mod error;
mod filter;
pub mod generator;
#[cfg(any(feature = "gis", feature = "gis-tokio-0_2"))]
mod gis;
mod order_item;
mod value;

pub use db::Connection;
pub use error::{ArysnError as Error, Optional, Result};
#[cfg(any(feature = "gis", feature = "gis-tokio-0_2"))]
pub use gis::Point;

pub mod prelude {
    pub use super::builder::BuilderTrait;
    pub use super::db::connect;
    pub use super::filter::Filter;
    pub use super::order_item::OrderItem;
    pub use super::value::ToSqlValue;
}
