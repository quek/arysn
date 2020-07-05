use anyhow::Result;
use tokio_postgres::{Client, NoTls};

pub async fn connect() -> Result<Client> {
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
    Ok(client)
}
