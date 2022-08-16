use crate::error::Result;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use deadpool_postgres::Transaction as PoolTransaction;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, NoTls, ToStatement, Transaction as ClientTransaction};

pub async fn connect<'a>() -> Result<Connection<'a>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    #[cfg(debug_assertions)]
    client.execute("SET log_statement = 'all'", &[]).await?;
    client.execute("SET TIME ZONE 'Japan'", &[]).await?;
    Ok(Connection::ClientConnection(client))
}

pub enum Connection<'a> {
    ClientConnection(Client),
    ClientTransaction(ClientTransaction<'a>),
    PoolConnection(Object<Manager>),
    PoolTransaction(PoolTransaction<'a>),
}

impl<'a> Connection<'a> {
    pub async fn commit(self) -> Result<()> {
        match self {
            Connection::ClientConnection(_) => {
                unimplemented!();
            }
            Connection::ClientTransaction(x) => {
                x.commit().await?;
            }
            Connection::PoolConnection(_) => {
                unimplemented!();
            }
            Connection::PoolTransaction(x) => {
                x.commit().await?;
            }
        }
        Ok(())
    }

    pub async fn execute<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> std::result::Result<u64, tokio_postgres::Error>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            Connection::ClientConnection(x) => x.execute(statement, params).await,
            Connection::ClientTransaction(x) => x.execute(statement, params).await,
            Connection::PoolConnection(x) => x.execute(statement, params).await,
            Connection::PoolTransaction(x) => x.execute(statement, params).await,
        }
    }

    pub async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> std::result::Result<Vec<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            Connection::ClientConnection(x) => x.query(statement, params).await,
            Connection::ClientTransaction(x) => x.query(statement, params).await,
            Connection::PoolConnection(x) => x.query(statement, params).await,
            Connection::PoolTransaction(x) => x.query(statement, params).await,
        }
    }

    pub async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> std::result::Result<tokio_postgres::Row, tokio_postgres::Error>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            Connection::ClientConnection(x) => x.query_one(statement, params).await,
            Connection::ClientTransaction(x) => x.query_one(statement, params).await,
            Connection::PoolConnection(x) => x.query_one(statement, params).await,
            Connection::PoolTransaction(x) => x.query_one(statement, params).await,
        }
    }

    pub async fn query_opt<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> std::result::Result<Option<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            Connection::ClientConnection(x) => x.query_opt(statement, params).await,
            Connection::ClientTransaction(x) => x.query_opt(statement, params).await,
            Connection::PoolConnection(x) => x.query_opt(statement, params).await,
            Connection::PoolTransaction(x) => x.query_opt(statement, params).await,
        }
    }

    pub async fn transaction<'b>(&'b mut self) -> Result<Connection<'b>> {
        let transaction = match self {
            Connection::ClientConnection(x) => {
                Connection::ClientTransaction(x.transaction().await?)
            }
            Connection::ClientTransaction(x) => {
                Connection::ClientTransaction(x.transaction().await?)
            }
            Connection::PoolConnection(x) => Connection::PoolTransaction(x.transaction().await?),
            Connection::PoolTransaction(x) => Connection::PoolTransaction(x.transaction().await?),
        };
        Ok(transaction)
    }
}
