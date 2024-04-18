use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MeiliConfig{
    api_url: String,    //Meili's endpoint

    #[serde(skip_serializing_if = "Option::is_none")]
    admin_api_key: Option<String>,      // key to interact with Meili's APIs
}

impl MeiliConfig{
    pub fn get_api_url(&self) -> &str{
        &self.api_url
    }
    pub fn get_admin_api_key(&self) -> &Option<String>{
        &self.admin_api_key
    }

}