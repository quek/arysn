#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use arysn_macro::defar;
    use tokio_postgres::{Error, NoTls};

    defar!(Neko);

    #[tokio::test]
    async fn it_works() -> Result<(), Error> {
        let neko = Neko {
            name: "こねら".to_string(),
        };
        println!("{}!!!!", &neko.name);
        assert_eq!("こねら", &neko.name);

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");

        // Connect to the database.
        let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        client.execute("SET log_statement = 'all'", &[]).await?;
        client.execute("SET TIME ZONE 'Japan'", &[]).await?;
        // Now we can execute a simple statement that just returns its parameter.
        let rows = client.query("SELECT $1::TEXT", &[&"hello world"]).await?;

        // And then check that we got back the same string we sent over.
        let value: &str = rows[0].get(0);
        assert_eq!(value, "hello world");
        Ok(())
    }
}
