use std::sync::{Arc, OnceLock};

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;
use crate::data_source::postgres::Postgres;
use crate::data_source::SupportedDataSource::*;
use crate::meili::{EventMessage, MeiliSearchService};
use crate::meili::index_setting::IndexSetting;
use crate::meili::meili_config::MeiliConfig;
use crate::meili::meili_enum::Event::*;

pub struct Synchronizer<'a> {
     meili_config: MeiliConfig,
     data_source_config: DataSourceConfig,
     synchronize_tables: Vec<IndexSetting>,
     sender: Sender<EventMessage>,
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
                    Some(index_setting) => meili_service.handle_event(event, index_setting).await,
                    None => println!("Table {} was not registered for changes !!!", event.index_name)
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
    async fn initialize_data_source(source_config: DataSourceConfig, index: IndexSetting) -> impl DataSource{
        match source_config.get_data_source() {
            Postgres => {
                let postgres = Postgres::new(index, source_config);
                if postgres.get_version().await < 12.0 {
                    panic!("Not support Postgres version below 12");
                }
                println!("Postgres version: {}", postgres.get_version().await);
                postgres
            }
        }
    }

   pub async fn sync(&'static self){
       let synchronize_tables =  &self.synchronize_tables;
       let meili_config = &self.meili_config;

       for index_setting in synchronize_tables{
           let table = index_setting.clone();
           let source_config = self.data_source_config.clone();

           let query_limit = meili_config.get_upload_size().unwrap();
           // query all data and start event listener
           let source = Arc::new(Self::initialize_data_source(source_config, table).await);
           let mv_source = source.clone();
           tokio::spawn(async move {
               let all_data = mv_source.get_full_data(query_limit.clone()).await;
               let event_message = EventMessage{
                   event_type: Insert,
                   index_name: index_setting.get_index_name().to_string(),
                   payload: Arc::new(all_data)
               };
               self.meili_service.handle_event(event_message, index_setting.clone()).await;
           });
           source.start_event_notifier(index_setting.clone(), self.data_source_config.clone(), self.sender.clone()).await;
       }
   }


}