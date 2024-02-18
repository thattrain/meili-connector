
pub mod data_source_enum;
pub mod data_source_setting;
pub mod postgres;

//todo: define a trait for specific datasource to implement

pub trait DataSource{
     async fn get_full_data(&self) -> String;

}

