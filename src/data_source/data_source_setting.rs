use serde::{Deserialize, Serialize};
use crate::data_source::SupportedDataSource;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceConfig{
    source_type:  SupportedDataSource,
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

impl DataSourceConfig {
    pub fn get_source_type(&self) -> &SupportedDataSource{
        &self.source_type
    }
    pub fn get_host(&self) -> &String{
        &self.host
    }
    pub fn get_port(&self) -> &u16{
        &self.port
    }
    pub fn get_username(&self) -> &String{
        &self.username
    }
    pub fn get_password(&self) -> &String {
        &self.password
    }
    pub fn get_database(&self) -> &String{
        &self.database
    }


}

