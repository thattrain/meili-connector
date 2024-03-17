use std::sync::{Arc, OnceLock};

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;
use crate::data_source::postgres::PostgresSource;
use crate::meili::{EventMessage, MeiliSearchService};
use crate::meili::index_setting::IndexSetting;
use crate::meili::meili_config::MeiliConfig;
use crate::meili::meili_enum::Event::*;
use crate::data_source::data_source_enum::SupportedDataSource::*;


pub struct Synchronizer<'a> {
     meili_config: MeiliConfig,
     data_source_config: DataSourceConfig,
     synchronize_tables: Vec<IndexSetting>,
     sender: Sender<EventMessage>,         // can clone as many senders as we want
     meili_service: &'a MeiliSearchService
}


impl Synchronizer<'static>{

    pub fn get_synchronizer<'a>(meili_config: MeiliConfig, data_source_config: DataSourceConfig, synchronize_tables: Vec<IndexSetting>) ->  &'static Synchronizer<'a> {
        let (tx,mut rx) = mpsc::channel::<EventMessage>(32);
        let mut meili_service: &'static MeiliSearchService = MeiliSearchService::get_meili_service(meili_config.clone());
        let index_setting_list = synchronize_tables.clone();

        tokio::spawn(async move {
            println!("Start event lister for Meilisearch....");
            while let Some(event) = rx.recv().await{
                let index_setting = Self::get_index_setting_by_name(&event.index_name, &index_setting_list);
                match index_setting {
                    Some(index_setting) => {
                        // println!("Receive event type: {:?} - Payload type: {} - table name: {}", event.event_type, event.payload.len(), event.index_name);
                        meili_service.handle_event(event, index_setting).await;
                    },
                    None => {
                        println!("Table {} was not registered for changes !!!", event.index_name)
                    }
                }
            }
        });
        static INSTANCE: OnceLock<Synchronizer> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Synchronizer{
                meili_config,
                data_source_config,
                synchronize_tables,
                sender: tx,
                meili_service
            }
        })
    }

    fn get_index_setting_by_name(index_name: &String, synchronize_tables: &Vec<IndexSetting>) -> Option<IndexSetting>{
        let index_setting = synchronize_tables.clone()
            .into_iter()
            .find(|index_setting| index_setting.get_index_name() == index_name);
        return index_setting;
    }

   pub async fn sync(&'static self){
       let synchronize_tables =  &self.synchronize_tables;
       let meili_config = &self.meili_config;

       for index_setting in synchronize_tables{
           let table = Arc::new(index_setting.clone());
           let test_table = index_setting.clone();
           let data_source_config = Arc::new(self.data_source_config.clone());

           // start event listener and query all data
           let query_limit = meili_config.get_upload_size().unwrap();
           tokio::spawn(async move {
               let postgres_source = PostgresSource::new(Arc::clone(&table), Arc::clone(&data_source_config));
               let all_data = postgres_source.get_full_data(query_limit.clone()).await;
               let event_message = EventMessage{
                   event_type: Insert,
                   index_name: test_table.get_index_name().to_string(),
                   payload: Arc::new(all_data)
               };
               self.meili_service.handle_event(event_message, test_table).await;
           });
           PostgresSource::start_event_notifier(index_setting.clone(), self.data_source_config.clone(), self.sender.clone()).await;
       }
    }


}