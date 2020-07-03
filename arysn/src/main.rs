#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use arysn_macro::defar;
    use tokio_postgres::{Error, NoTls};

    defar!(User);

    #[tokio::test]
    async fn it_works() -> Result<(), Error> {
        let user = User {
            id: 1,
            name: "こねら".to_string(),
            title: None,
        };
        println!("{}!!!!", &user.name);
        assert_eq!("こねら", &user.name);

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
        let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        client.execute("SET log_statement = 'all'", &[]).await?;
        client.execute("SET TIME ZONE 'Japan'", &[]).await?;

        let rows = client.query("SELECT $1::TEXT", &[&"hello world"]).await?;
        let value: &str = rows[0].get(0);
        assert_eq!(value, "hello world");

        let rows = client
            .query("SELECT id, name FROM users where id=$1", &[&1i64])
            .await?;
        println!("rows {:?}", &rows);
        let id: i64 = rows[0].get(0);
        assert_eq!(id, 1);
        let value: &str = rows[0].get(1);
        assert_eq!(value, "ユーザ1");

        Ok(())
    }
}
