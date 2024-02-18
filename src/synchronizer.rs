use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;
use crate::data_source::postgres::PostgresSource;
use crate::meili::index_setting::IndexSetting;
use crate::meili::meili_config::MeiliConfig;

// This module sync database with meilisearch, ex: read from database and upload to meilisearch
// this is where data_source and meili module interact with each other
pub struct Synchronizer<'a> {
     meili_config: &'a MeiliConfig,
     data_source_config: &'a DataSourceConfig,
     synchronize_tables: &'a Vec<IndexSetting>
}

impl Synchronizer<'_>{
    pub fn new<'a>(meili_config: &'a MeiliConfig, data_source_config: &'a DataSourceConfig, synchronize_tables: &'a Vec<IndexSetting>) -> Synchronizer<'a> {
        Synchronizer{
            meili_config,
            data_source_config,
            synchronize_tables,
        }
    }
   pub async fn sync(&self){
        let postgres = PostgresSource::new(self.synchronize_tables, self.data_source_config);
        postgres.get_full_data().await;
    }
}