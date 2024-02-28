use meilisearch_sdk::{Client};
use serde_json::Value;
use crate::meili::index_setting::IndexSetting;
use crate::meili::meili_config::MeiliConfig;

pub mod meili_config;
pub mod index_setting;
pub mod meili_enum;

//todo: implement interaction with meilisearch instance
pub struct MeiliSearchService{
     meili_client: Client,
}
impl MeiliSearchService{
    pub fn new(meili_config: &MeiliConfig) -> Self{
        let mut client = Client::new(meili_config.get_api_url(), meili_config.get_admin_api_key().as_ref());
         MeiliSearchService {
            meili_client: client
        }
    }

    fn get_meili_client(&self) -> &Client {
        &self.meili_client
    }

     pub async fn add_documents(&self, index_setting: &IndexSetting, documents: &Vec<Value>) {
        let index = &self.get_meili_client().index(index_setting.get_index_name());
        // let task = index.add_documents_ndjson(documents, Some(index_setting.get_primary_key().as_str()));
        // &self.get_meili_client().wait_for_task(task, None, None).await.unwrap();
    }
}
