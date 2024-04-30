use std::sync::{Arc, OnceLock};
use std::time::Duration;

use log::{debug, error, info};
use tokio::{task, time};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Sleep;

use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;
use crate::data_source::postgres::Postgres;
use crate::data_source::SupportedDataSource::*;
use crate::meili::{EventMessage, MeiliSearchService};
use crate::meili::Event::*;
use crate::meili::index_setting::IndexSetting;
use crate::meili::meili_config::MeiliConfig;

const SWAP_INDEX_PREFIX: &str = "_swap";


pub struct Synchronizer<'a> {
     meili_config: MeiliConfig,
     data_source_config: DataSourceConfig,
     synchronize_tables: Vec<IndexSetting>,
     meili_service: &'a MeiliSearchService,
     runtime: Runtime
}


impl Synchronizer<'static>{
    pub fn get_synchronizer<'a>(meili_config: MeiliConfig, data_source_config: DataSourceConfig, synchronize_tables: Vec<IndexSetting>, runtime: Runtime) ->  &'static Synchronizer<'a> {
        let meili_service: &'static MeiliSearchService = MeiliSearchService::get_meili_service(&meili_config);
        static INSTANCE: OnceLock<Synchronizer> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Synchronizer{
                meili_config,
                data_source_config,
                synchronize_tables,
                meili_service,
                runtime
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
                info!("Postgres version: {}", postgres.version().await);
                postgres
            }
        }
    }

     fn start_meili_listener(&'static self, mut rx: Receiver<EventMessage>){
         tokio::spawn(async move {
             while let Some(event) = rx.recv().await{
                 let index_setting = self.get_index_setting_by_name(&event.index_name);
                 match index_setting {
                     Some(index_setting) => self.meili_service.handle_event(event, index_setting).await,
                     None => debug!("Table {} was not registered for changes !!!", event.index_name)
                 }
             }
         });
     }

    async fn sync_task(&'static self, index: &'static IndexSetting, sender: Sender<EventMessage>){
        info!("Start syncing table: {} to Meilisearch", index.get_index_name());
        //create or update meilisearch index settings before sync
        self.meili_service.initialize_index(index).await;

        // query all data and start event listener
        let source = Arc::new(self.initialize_data_source(index).await);
        let total_records = source.total_record().await;
        info!("Total record need to be sync: {} from table: {}", total_records, index.get_index_name());
        let limit = index.get_limit();
        let mut current_page = 0;
        let mut offset = 0;
        let total_page = (total_records as f64 / limit as f64).ceil() as usize;
        while current_page < total_page {
            let data_source = Arc::clone(&source);
            tokio::spawn(async move {
                let data = data_source.get_data(limit, offset).await;
                let event_message = EventMessage {
                    event_type: Insert,
                    index_name: index.get_index_name().to_string(),
                    payload: Arc::new(data)
                };
                self.meili_service.handle_event(event_message, index).await;
            });
            offset += limit;
            current_page += 1;
        }

        info!("Sync table {} successfully to Meilisearch!", index.get_index_name());
        source.start_event_notifier(index, &self.data_source_config, sender).await;
    }

    async fn refresh_task(&'static self, index: &'static IndexSetting, sender: Sender<EventMessage>){
        let swap_index_uuid = Arc::new(format!("{}{}", {index.get_index_name()}, SWAP_INDEX_PREFIX));
        let IndexSetting { primary_key, sync_fields, limit, meili_setting, .. } = index;
        let swap_index_setting = IndexSetting{
            index_name: swap_index_uuid.to_string(),
            primary_key: primary_key.to_string(),
            sync_fields: sync_fields.clone(),
            limit: limit.clone(),
            meili_setting: meili_setting.clone()
        };
        if !self.meili_service.is_index_exist(index.get_index_name()).await {
            error!("Index {} does not exist - Cannot refresh data by swap index !", index.get_index_name());
            return;

        }
        self.meili_service.create_index(&swap_index_setting).await;
        let source = Arc::new(self.initialize_data_source(index).await);
        let mv_source = Arc::clone(&source);
        let sync_task = tokio::spawn(async move {
            let total_records = mv_source.total_record().await;
            info!("Total record need to be sync: {} from table: {}", total_records, index.get_index_name());
            let limit = index.get_limit();
            let mut current_page = 0;
            let mut offset = 0;
            let total_page = (total_records as f64 / limit as f64).ceil() as usize;
            let swap_index = Arc::new(swap_index_setting);
            while current_page < total_page{
                let data_source = Arc::clone(&mv_source);
                let setting = Arc::clone(&swap_index);
                tokio::spawn(async move {
                    let data = data_source.get_data(limit, offset).await;
                    let event_message = EventMessage{
                        event_type: Insert,
                        index_name: setting.get_index_name().to_string(),
                        payload: Arc::new(data)
                    };
                    self.meili_service.handle_event(event_message, &setting).await;
                }).await.unwrap();
                offset += limit;
                current_page += 1;
            }
        });

        if sync_task.await.is_ok() {
            let swap_result = self.meili_service.swap_index(index.get_index_name().to_string(), swap_index_uuid.to_string()).await;
            match swap_result {
                Ok(task_info) => {
                    info!("Refresh data by swap index success for index: {}", index.get_index_name());
                    debug!("{:?}", task_info);
                    self.meili_service.drop_index(&swap_index_setting.get_index_name()).await;
                    // start event listener when full data from a table is in sync
                    source.start_event_notifier(index, &self.data_source_config, sender).await;
                },
                Err(error) => {
                    error!("Error while swapping index: {} - Error: {}", index.get_index_name(), error);
                    return;
                }
            }
        }
    }

   pub async fn sync(&'static self){
       //todo: check status from last run
       let (tx, rx) = mpsc::channel::<EventMessage>(32);
       self.start_meili_listener(rx);
       let mut task_handles = Vec::with_capacity(self.synchronize_tables.len());

       for index_setting in &self.synchronize_tables{
           let sender = tx.clone();
           task_handles.push(self.runtime.spawn(self.sync_task(index_setting, sender)));
       }
        for handle in task_handles {
            futures::executor::block_on(handle).unwrap();
        }
   }


    pub async fn refresh(&'static self){
        info!("Refresh data by swap index");
        let (tx, rx) = mpsc::channel::<EventMessage>(32);
        self.start_meili_listener(rx);
        let mut task_handles = Vec::with_capacity(self.synchronize_tables.len());

        for index in &self.synchronize_tables{
            let sender = tx.clone();
            task_handles.push(self.runtime.spawn(self.refresh_task(index, sender)));
        }

        for handle in task_handles {
            futures::executor::block_on(handle).unwrap();
        }
    }

    pub async fn status(&'static self){
        for index in &self.synchronize_tables{
            let source = Arc::new(self.initialize_data_source(index).await);
            let source_records = source.total_record().await;
            let meili_records = self.meili_service.count_document(index.get_index_name()).await;

            if source_records == meili_records{
                info!("Table {} is consistent with Meilisearch instance - Documents count: {}", index.get_index_name(), source_records)
            }else {
                error!("Table {} is inconsistent with Meilisearch - Source records: {} - Meilisearch documents: {}"
                    , index.get_index_name()
                    , source_records
                    , meili_records);
            }
        }
    }

}