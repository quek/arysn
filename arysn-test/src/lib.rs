#[cfg(feature = "with-tokio-0_2")]
extern crate postgres_types_0_1 as postgres_types;
#[cfg(feature = "with-tokio-0_2")]
extern crate tokio_0_2 as tokio;
#[cfg(feature = "with-tokio-0_2")]
extern crate tokio_postgres_0_5 as tokio_postgres;

#[cfg(feature = "with-tokio-1_x")]
extern crate postgres_types_0_2 as postgres_types;
#[cfg(feature = "with-tokio-1_x")]
extern crate tokio_1_x as tokio;
#[cfg(feature = "with-tokio-1_x")]
extern crate tokio_postgres_0_7 as tokio_postgres;

pub mod generated;
