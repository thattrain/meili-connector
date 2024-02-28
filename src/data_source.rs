use serde_json::Value;

pub mod data_source_enum;
pub mod data_source_setting;
pub mod postgres;
pub mod deserializer;

//todo: define a trait for specific datasource to implement

pub trait DataSource{
     async fn get_total_record_num(&self) -> i64;
     async fn get_full_data(&self, size: i64) -> Vec<Value>;

}

