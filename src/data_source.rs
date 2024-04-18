use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::data_source::data_source_setting::DataSourceConfig;
use crate::meili::EventMessage;
use crate::meili::index_setting::IndexSetting;

pub mod data_source_setting;
pub mod postgres;
pub mod deserializer;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SupportedDataSource{
     Postgres,
}


#[async_trait]
pub trait DataSource: Send{
     async fn version(&self) -> f32;
     async fn total_record(&self) -> i64;
     async fn get_data(&self, limit: i64, offset: i64) -> Vec<Value>;
     async fn start_event_notifier(&self, index_setting: &IndexSetting, data_source_config: &DataSourceConfig, event_sender: Sender<EventMessage>);
}





