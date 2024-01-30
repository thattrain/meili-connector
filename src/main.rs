use tokio_postgres::{NoTls, Error};
use crate::config::*;
use crate::data_source::data_source_enum::SupportedDataSource;
use crate::data_source::DataSource;
use crate::data_source::postgres::Postgres;

mod meili;
mod config;
mod data_source;
mod synchronizer;

#[tokio::main]
async  fn main() -> Result<(), Error> {
    let config = Config::read_config("config.yaml".to_string());
    let data_source_config = config.get_data_source();
    let source_type = data_source_config.get_data_source();
    match source_type {
        SupportedDataSource::PostgresSQL => {
            let postgres_source = Postgres::new(&data_source_config).await;
            postgres_source.get_full_data().await.expect("Failed to get data from Postgres");

        }
    };

    Ok(())

}
