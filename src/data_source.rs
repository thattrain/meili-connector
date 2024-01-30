use tokio_postgres::{Error};

pub mod data_source_enum;
pub mod data_source_setting;
pub mod postgres;

pub trait DataSource{
     async fn get_full_data(&self) -> Result<(), Error>;

}

