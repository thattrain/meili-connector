use std::sync::Arc;

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

       for index in synchronize_tables{
           let table = Arc::new(index);
           let postgres_source = PostgresSource::new(table, self.data_source_config.clone());

           let all_data = postgres_source.get_full_data(4).await;
           for record in &all_data{
               println!("Record: {:?}", record);
           }
       }


    }
}