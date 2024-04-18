use meilisearch_sdk::{Settings};

use serde::{Deserialize, Serialize};

// - If displayed_fields and searchable_fields empty all fields
//in the index will be both displayed and searchable by default
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexSetting{

/*  Associate with a table name in synced database */
    index_name: String,

/*  Associate with primary key of the table */
    primary_key: String,

/*  List of fields from data source to sync */
    #[serde(skip_serializing_if = "Option::is_none")]
    sync_fields: Option<Vec<String>>,

/*  Number of records to be queried and uploaded to Meili at an interval */
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i64>,

/*  Meilisearch settings */
    #[serde(skip_serializing_if = "Option::is_none")]
    meili_setting: Option<Settings>,
}


impl IndexSetting{
    pub fn get_index_name(&self) -> &String{
        return &self.index_name;
    }
    pub fn get_primary_key(&self) -> &String{
        return  &self.primary_key;
    }
    pub fn get_sync_fields(&self) -> &Option<Vec<String>>{
        return &self.sync_fields
    }
    pub fn get_limit(&self) -> i64{
        return match self.limit {
            Some(limit) => { limit },
            None => { 10000 }
        };
    }
    pub fn get_meili_setting(&self) -> &Option<Settings>{
        return &self.meili_setting;
    }
}
