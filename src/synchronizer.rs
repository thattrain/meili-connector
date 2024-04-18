use std::sync::{Arc, OnceLock};

use tokio::sync::mpsc;

use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;
use crate::data_source::postgres::Postgres;
use crate::data_source::SupportedDataSource::*;
use crate::meili::{EventMessage, MeiliSearchService};
use crate::meili::index_setting::IndexSetting;
use crate::meili::meili_config::MeiliConfig;
use crate::meili::Event::*;

pub struct Synchronizer<'a> {
     meili_config: MeiliConfig,
     data_source_config: DataSourceConfig,
     synchronize_tables: Vec<IndexSetting>,
     meili_service: &'a MeiliSearchService
}


impl Synchronizer<'static>{
    pub fn get_synchronizer<'a>(meili_config: MeiliConfig, data_source_config: DataSourceConfig, synchronize_tables: Vec<IndexSetting>) ->  &'static Synchronizer<'a> {
        let mut meili_service: &'static MeiliSearchService = MeiliSearchService::get_meili_service(meili_config.clone());
        static INSTANCE: OnceLock<Synchronizer> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Synchronizer{
                meili_config,
                data_source_config,
                synchronize_tables,
                meili_service
            }
        })
    }
    fn get_index_setting_by_name(&self, index_name: &str) -> Option<&IndexSetting>{
        let index_setting = self.synchronize_tables.as_slice()
            .into_iter()
            .find(|index_setting| index_setting.get_index_name() == index_name);
        return index_setting;
    }
    async fn initialize_data_source(&self, index: &IndexSetting) -> impl DataSource {
        match &self.data_source_config.get_source_type() {
            Postgres => {
                let postgres = Postgres::new(index.clone(),  self.data_source_config.clone());
                if postgres.version().await < 12.0 {
                    panic!("Not support Postgres version below 12");
                }
                println!("Postgres version: {}", postgres.version().await);
                postgres
            }
        }
    }

   pub async fn sync(&'static self){
       let (tx,mut rx) = mpsc::channel::<EventMessage>(32);
       tokio::spawn(async move {
           while let Some(event) = rx.recv().await{
               let index_setting = self.get_index_setting_by_name(&event.index_name);
               match index_setting {
                   Some(index_setting) => self.meili_service.handle_event(event, index_setting).await,
                   None => println!("Table {} was not registered for changes !!!", event.index_name)
               }
           }
       });

       for index_setting in &self.synchronize_tables{

           //create meilisearch index before sync
            self.meili_service.initialize_index(index_setting).await;

           // query all data and start event listener
           let source = Arc::new(self.initialize_data_source(index_setting).await);
           let total_records = source.total_record().await;
           println!("Total record need to be sync: {} from table: {}", total_records, index_setting.get_index_name());
           let limit = index_setting.get_limit();
           let mut current_page = 0;
           let mut offset = 0;
           let total_page = if total_records % limit == 0 {
               (total_records / limit) as f64
           }else {
               ((total_records / limit) as f64).round()
           };
           while current_page < total_page as i64{
               let data_source = Arc::clone(&source);
               tokio::spawn(async move {
                   let data = data_source.get_data(limit, offset).await;
                   let event_message = EventMessage{
                       event_type: Insert,
                       index_name: index_setting.get_index_name().to_string(),
                       payload: Arc::new(data)
                   };
                   self.meili_service.handle_event(event_message, index_setting).await;
               });
               offset += limit;
               current_page += 1;
           }

           // start event listener when full data from a table is in sync
           source.start_event_notifier(index_setting, &self.data_source_config, tx.clone()).await;
       }
   }


}