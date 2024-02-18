use serde_json::Value;
use tokio_postgres::NoTls;
use crate::config::*;
use crate::synchronizer::Synchronizer;

mod meili;
mod config;
mod data_source;
mod synchronizer;

#[tokio::main]
async  fn main(){
    // let config = Config::read_config("config.yaml".to_string());
    // let data_source_config = config.get_data_source();
    // let synchronize_tables = config.get_synchronize_tables();
    // let meili_config = config.get_meili_config();
    //
    // let synchronizer = Synchronizer::new(meili_config, data_source_config, synchronize_tables);
    // synchronizer.sync().await;

    // Replace with your actual PostgreSQL connection string
    let conn_str = "postgresql://postgres:postgres@localhost:5432/test_meilisearch";
    let (client, connection) = tokio_postgres::connect(conn_str, NoTls).await.expect("ssadsadsad");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Replace the query with your actual SQL query
    let query = "SELECT * FROM users";
    let statement = client.prepare(query).await.expect("ASdasdasdsadsa");

    // Execute the query and fetch the results
    let rows = client.query(query, &[]).await.expect("SdasdasdASDSA");

    // Process each row
    for row in rows {
        // Process the row dynamically
        for (i, column) in row.columns().iter().enumerate() {
            let value: i32 = row.try_get(i).unwrap();
            println!("Column {}: {}", column.name(), value);
        }
    }


}
