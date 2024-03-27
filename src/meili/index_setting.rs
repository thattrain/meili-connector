use std::collections::HashMap;
use std::os::macos::raw::stat;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};

// - If displayed_fields and searchable_fields empty all fields
//in the index will be both displayed and searchable by default
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexSetting{
/*  Associate with a table name in synced database */
    index_name: String,

/*  Associate with primary key of the table */
    primary_key: String,

/*  List of fields to sync */
    // #[serde_as(as = "Option<Vec<String>>")]
    sync_fields: Option<Vec<String>>,

/*  List contains displayed fields    */
    displayed_fields: Option<Vec<String>>,

/*  List contains searchable fields, this would impact on the search result and ranking order.
    Order of elements in this list will determine impact on relevancy from the most impact to the least.    */
    searchable_fields: Option<Vec<String>>,

/*  List of synonyms map by string - string   */
    synonyms: Option<Vec<HashMap<String,String>>>,

/*  List of distinct attributes in the index  */
    distinct_attributes: Option<Vec<String>>,

/*  Meili uses this rules of sort matching result so the most relevant documents stay on top.
    Order of the elements in this list will determine from the most impacted rules to the least.    */
    ranking_rules: Option<Vec<String>>,

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
}
