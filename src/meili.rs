use std::sync::{Arc, OnceLock};

use log::{debug, error, info};
use meilisearch_sdk::{Client, Error, ErrorCode, Index, SwapIndexes, TaskInfo};
use serde_json::Value;

use crate::meili::index_setting::IndexSetting;
use crate::meili::meili_config::MeiliConfig;

pub mod meili_config;
pub mod index_setting;



#[derive(Debug)]
pub enum Event {
    Insert,
    Update,
    Delete
}

pub struct MeiliSearchService {
     meili_client: Client,
}
#[derive(Debug)]
pub struct EventMessage {
    pub event_type: Event,
    pub index_name: String,
    pub payload: Arc<Vec<Value>>
}


impl MeiliSearchService{
    pub fn get_meili_service(meili_config: &MeiliConfig) -> &'static MeiliSearchService{
        static INSTANCE: OnceLock<MeiliSearchService> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let client = Client::new(meili_config.get_api_url(), meili_config.get_admin_api_key().as_ref());
            MeiliSearchService {
                meili_client: client,
            }
        })
    }

     fn get_meili_client(&self) -> Option<&Client>{
        if Some(self).is_none(){
            return None;
        }
       Some(&self.meili_client)
    }



    //region handle meili indexes
    pub async fn is_index_exist(&self, index_name: &str) -> bool {
        let client = self.get_meili_client();

        if client.is_none() {
            panic!("Can not create connection to Meilisearch instance");
        }

        match client.unwrap().get_index(index_name).await {
            Ok(_index) => {
                true
            },
            Err(err) => match err {
                Error::Meilisearch(ref error) => {
                    if error.error_code == ErrorCode::IndexNotFound{
                        false
                    }else {
                        panic!("Unknown error: {:?}", err);
                    }
                },
                _ => {
                    panic!("Unknown error: {:?}", err);
                }
            }
        }
    }

    pub async fn create_index(&self, index_setting: &IndexSetting) -> Index{
        let meili_client = self.get_meili_client();
        if meili_client.is_none() {
            panic!("Can not create connection to Meilisearch instance");
        }

        let index_name = index_setting.get_index_name().as_str();
        info!("Create new Meilisearch index: '{}'", index_name);
        let client = meili_client.unwrap();
        let index = client
            .create_index(index_name, Some(index_setting.get_primary_key()))
            .await
            .unwrap()
            .wait_for_completion(client, None, None)
            .await
            .unwrap()
            .try_make_index(&client)
            .unwrap();

        match index_setting.get_meili_setting() {
            Some(meili_settings) => { index.set_settings(meili_settings).await.unwrap(); },
            None => { debug!("None Meilisearch settings were set on index: {}", index_setting.get_index_name()) }
        }
        index
    }

    pub async fn drop_index(&self, index_name: &str) {
        if !self.is_index_exist(index_name).await {
            info!("Index {} is not exist - Can not drop", index_name);
            return;
        }

        let meili_client = self.get_meili_client();
        match meili_client {
            Some(client) => {
                let task = client.delete_index(index_name).await;
                match task {
                    Ok(task_info) => {
                        info!("Drop Meilisearch index: {}", index_name);
                        debug!("{:?}", task_info);

                    },
                    Err(error) => error!("Error while drop index: {}", error)
                }
            },
            None => error!("Can not initialize connection to Meilisearch")
        }
    }

    pub async fn swap_index(&self, original_index: String, swap_index: String) -> Result<TaskInfo, Error>{
        let meili_client = self.get_meili_client();
        if meili_client.is_none() {
            panic!("Can not initialize connection to Meilisearch");
        }

        info!("Swap index {} and {}", original_index, swap_index);
        let client = meili_client.unwrap();
        let task = client.swap_indexes(
            [&SwapIndexes{
                indexes: (
                    original_index,
                    swap_index
                ),
            }])
            .await;
        task
    }

    pub async fn initialize_index(&self, index_setting: &IndexSetting) -> Index{
       let index_name = index_setting.get_index_name().as_str();
       let client = self.get_meili_client();

       if client.is_none() {
           panic!("Can not create connection to Meilisearch instance");
       }

        if self.is_index_exist(index_name).await {
            let index = client.unwrap().get_index(index_name).await.unwrap();
            match index_setting.get_meili_setting() {
                Some(meili_settings) => { index.set_settings(meili_settings).await.unwrap(); },
                None => { debug!("None Meilisearch settings were set on index: {}", index_setting.get_index_name()) }
            }
            info!("Meilisearch index: '{}' already exits", index_name);
            index
        }else {
            let index = self.create_index(index_setting).await;
            index
        }
    }
    //endregion handle meili indexes



    // region handle meili documents
    pub async fn handle_event(&self, event_message: EventMessage, index_setting: &IndexSetting){
        let documents = event_message.payload;
        // println!("Record: {:?}", &documents);
        match event_message.event_type {
            Event::Insert => {
               match self.add_documents(index_setting, documents.as_ref()).await {
                   Ok(task_info) => debug!("Insert: {:?}", task_info),
                   Err(err) => error!("Error when insert to meili: {}", err)
               }
            },
            Event::Update => {
                match self.update_document(index_setting, documents.as_ref()).await {
                    Ok(task_info) => debug!("Update: {:?}", task_info),
                    Err(err) => error!("Error when update to meili: {}", err)
                }
            },
            Event::Delete => {
                match self.delete_documents(index_setting, documents.as_ref()).await {
                    Ok(task_info) => debug!("Delete: {:?}", task_info),
                    Err(err) => error!("Error when delete to meili: {}", err)
                }
            }
        }
    }

    async fn add_documents(&self, index_setting: &IndexSetting, documents: &Vec<Value>) -> Result<TaskInfo, Error>{
        let meili_client = self.get_meili_client();
        match meili_client {
            Some(client) => {
                let index = client.get_index(index_setting.get_index_name()).await.unwrap();
                let pk_key = index_setting.get_primary_key().as_str();
                let mut ndjson = String::from("");
                for document in documents{
                    ndjson.push_str(serde_json::to_string(document).unwrap().as_str());
                    ndjson.push_str("\n");
                }
                let task_info =  index
                    .add_documents_ndjson(Box::leak(ndjson.into_boxed_str()).as_bytes(), Some(pk_key))
                    .await
                    .unwrap();
                Ok(task_info)
            },
            None => {
                error!("Can not initialize connection to Meilisearch");
                Err(Error::InvalidRequest)
            }
        }
    }

    async fn delete_documents(&self, index_setting: &IndexSetting, documents: &Vec<Value>) -> Result<TaskInfo, Error>{
        let meili_client = self.get_meili_client();
        match meili_client {
            Some(client) => {
                let index = client.get_index(index_setting.get_index_name()).await.unwrap();
                let pk_key = index_setting.get_primary_key();

                let document_ids: Vec<String> = documents
                    .iter()
                    .map(|document| document.get(pk_key).unwrap().to_string())
                    .collect::<Vec<_>>();

                let task_info = index
                    .delete_documents(&document_ids)
                    .await
                    .unwrap();
                Ok(task_info)
            },
            None => {
                error!("Can not initialize connection to Meilisearch");
                Err(Error::InvalidRequest)
            }
        }
    }

    async fn update_document(&self, index_setting: &IndexSetting, documents: &Vec<Value>) -> Result<TaskInfo, Error>{
        let meili_client = self.get_meili_client();
        match meili_client{
            Some(client) => {
                let index = client.get_index(index_setting.get_index_name()).await.unwrap();
                let pk_key = index_setting.get_primary_key().as_str();
                let mut ndjson = String::from("");
                for document in documents{
                    ndjson.push_str(serde_json::to_string(document).unwrap().as_str());
                    ndjson.push_str("\n");
                }
                let task_info = index
                    .update_documents_ndjson(Box::leak(ndjson.into_boxed_str()).as_bytes(), Some(pk_key))
                    .await
                    .unwrap();
                Ok(task_info)
            },
            None => {
                error!("Can not initialize connection to Meilisearch");
                Err(Error::InvalidRequest)
            }
        }
    }

    pub async fn count_document(&self, index_name: &str) -> usize{
        let meili_client = self.get_meili_client();
        match meili_client {
            Some(client) => {
                let index = client.get_index(index_name).await;
                match index {
                    Ok(meili_index) => {
                        let index_stats = meili_index.get_stats().await.unwrap();
                        let document_count = index_stats.number_of_documents;
                        document_count
                    },
                    Err(error) => {
                        error!("Error when counting documents from Meilisearch - index: {} - Error: {:?}", index_name, error);
                        panic!();
                    }
                }
            },
            None => {
                error!("Can not initialize connection to Meilisearch");
                panic!();
            }
        }
    }


    // endregion handle meili document
}
