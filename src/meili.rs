use std::sync::{Arc, OnceLock};

use meilisearch_sdk::{Client, Error, ErrorCode, Index, TaskInfo};
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
    pub fn get_meili_service(meili_config: MeiliConfig) -> &'static MeiliSearchService{
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
    pub async fn initialize_index(&self, index_setting: &IndexSetting) -> Index{
       let index_name = index_setting.get_index_name().as_str();
       let client = self.get_meili_client();

       if client.is_none() {
           panic!("Can not create connection to Meilisearch instance");
       }

       match client.unwrap().get_index(index_name).await {
           Ok(index) => {
               match index_setting.get_meili_setting() {
                   Some(meili_settings) => { index.set_settings(meili_settings).await.unwrap(); },
                   None => { println!("None Meilisearch settings were set on index: {}", index_setting.get_index_name()) }
               }
               println!("Meilisearch index: '{}' already exits", index_name);
               index
           },
           Err(err) => match err {
               Error::Meilisearch(ref error) => {
                    if error.error_code == ErrorCode::IndexNotFound{
                        println!("Create new meilisearch index: '{}'", index_name);
                        let meili_client = client.unwrap();
                        let index = meili_client
                            .create_index(index_name, Some(index_setting.get_primary_key()))
                            .await
                            .unwrap()
                            .wait_for_completion(meili_client, None, None)
                            .await
                            .unwrap()
                            .try_make_index(&meili_client)
                            .unwrap();

                        match index_setting.get_meili_setting() {
                            Some(meili_settings) => { index.set_settings(meili_settings).await.unwrap(); },
                            None => { println!("None Meilisearch settings were set on index: {}", index_setting.get_index_name()) }
                        }
                        index
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
    //endregion handle meili indexes



    // region handle meili documents
    pub async fn handle_event(&self, event_message: EventMessage, index_setting: &IndexSetting){
        let documents = event_message.payload;
        // println!("Record: {:?}", &documents);
        match event_message.event_type {
            Event::Insert => {
               match self.add_documents(index_setting, documents.as_ref()).await {
                   Ok(task_info) => println!("Insert: {:?}", task_info),
                   Err(err) => println!("Error when insert to meili: {}", err)
               }
            },
            Event::Update => {
                match self.update_document(index_setting, documents.as_ref()).await {
                    Ok(task_info) => println!("Delete: {:?}", task_info),
                    Err(err) => println!("Error when update to meili: {}", err)
                }
            },
            Event::Delete => {
                match self.delete_documents(index_setting, documents.as_ref()).await {
                    Ok(task_info) => println!("Update: {:?}", task_info),
                    Err(err) => println!("Error when delete to meili: {}", err)
                }
            }
        }
    }

    async fn add_documents(&self, index_setting: &IndexSetting, documents: &Vec<Value>) -> Result<TaskInfo, Error>{
         let meili_client = self.get_meili_client();
         if !meili_client.is_none() {
             let index = meili_client.unwrap().index(index_setting.get_index_name());
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
         }else {
             Err(Error::InvalidRequest)
         }


    }
    async fn delete_documents(&self, index_setting: &IndexSetting, documents: &Vec<Value>) -> Result<TaskInfo, Error>{
        let meili_client = self.get_meili_client();
        if !meili_client.is_none() {
            for document in documents{
                println!("Document need to be delete: {:?}", document);
            }
            let index = meili_client.unwrap().index(index_setting.get_index_name());
            let pk_key = index_setting.get_primary_key();

            let document_ids: Vec<String> = documents
                .iter()
                .map(|document| document.get(pk_key).unwrap().to_string())
                .collect::<Vec<_>>();

            for id in &document_ids{
                println!("Delete document id: {}", id);
            }

            let task_info = index
                .delete_documents(&document_ids)
                .await
                .unwrap();
            Ok(task_info)
        }else {
            Err(Error::InvalidRequest)
        }

    }

    async fn update_document(&self, index_setting: &IndexSetting, documents: &Vec<Value>) -> Result<TaskInfo, Error>{
        let meili_client = self.get_meili_client();
        if !meili_client.is_none() {
            let index = meili_client.unwrap().index(index_setting.get_index_name());
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
        }else {
            Err(Error::InvalidRequest)
        }
    }


    // endregion handle meili document
}
