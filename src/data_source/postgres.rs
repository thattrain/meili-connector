use tokio_postgres::{Client, NoTls, Error, row};
use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;

pub struct Postgres{
    client: Client
}

impl Postgres{
    pub async fn new(source_setting: &DataSourceConfig) -> Postgres {
        let mut config = format!("host={} port={} user={} dbname={} " ,
                                         source_setting.get_host(),
                                         source_setting.get_port(),
                                         source_setting.get_username(),
                                         source_setting.get_database());
        
        match source_setting.get_password() {
            Some(password) => {
                config.push_str(format!("password={}", password.trim()).as_str())
            }
            _ => {}
        }

        let (client, connection) = tokio_postgres::connect(&config.as_str(), NoTls)
            .await.expect("Can not create connection to Postgres host");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Postgres{
            client
        }
    }
}

impl DataSource for Postgres{
    async fn get_full_data(&self) -> Result<(), Error> {
        let rows = &self.client
            .query("SELECT username FROM users",&[])
            .await;
        println!("Size: {}", rows.as_ref().unwrap().len());

        Ok(())
    }
}