use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MeiliConfig{
    api_url: String,    //Meili's endpoint
    admin_api_key: Option<String>,      // key to interact with Meili's APIs
    upload_size: Option<i64>,            // number of records each sync task
}

impl MeiliConfig{
    pub fn get_api_url(&self) -> &str{
        &self.api_url
    }
    pub fn get_admin_api_key(&self) -> &Option<String>{
        &self.admin_api_key
    }
    pub fn get_upload_size(&self) -> &Option<i64>{
        &self.upload_size
    }

}