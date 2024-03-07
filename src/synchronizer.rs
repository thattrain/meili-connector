use std::sync::Arc;
use tokio::sync::Mutex;

use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;
use crate::data_source::postgres::PostgresSource;
use crate::meili::index_setting::IndexSetting;
use crate::meili::meili_config::MeiliConfig;

// This module sync database with meilisearch, ex: read from database and upload to meilisearch
// this is where data_source and meili module interact with each other
pub struct Synchronizer {
     meili_config: MeiliConfig,
     data_source_config: DataSourceConfig,
     synchronize_tables: Vec<IndexSetting>
}

impl Synchronizer{
    pub fn new(meili_config: MeiliConfig, data_source_config: DataSourceConfig, synchronize_tables: Vec<IndexSetting>) -> Synchronizer {
        Synchronizer{
            meili_config,
            data_source_config,
            synchronize_tables,
        }
    }
   pub async fn sync(self){
       let synchronize_tables = self.synchronize_tables;
       // let meili_config = self.meili_config;
       // let meili_service = MeiliSearchService::new()

       for index_setting in synchronize_tables{
           let table = Arc::new(index_setting.clone());
           let data_source_config = Arc::new(self.data_source_config.clone());

           // start event listener and query all data
           let meili_config = self.meili_config.clone();
           tokio::spawn(async move {
               let mut postgres_source = PostgresSource::new(Arc::clone(&table), Arc::clone(&data_source_config));
               let limit =  meili_config.get_upload_size().unwrap();
               let all_data = postgres_source.get_full_data(limit).await;
               for record in &all_data{
                   println!("Record: {:?}", record);
               }
           });
           PostgresSource::start_event_notifier(index_setting.clone(), self.data_source_config.clone()).await;
       }


    }
}