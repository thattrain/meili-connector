use serde::{Deserialize, Serialize};
use crate::data_source::data_source_enum::SupportedDataSource;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSourceConfig{
    source_type:  SupportedDataSource,
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    access_key: Option<String>,
    database: String,
}

impl DataSourceConfig {
    pub fn get_data_source(&self) -> &SupportedDataSource{
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
    pub fn get_password(&self) -> &Option<String> {
        &self.password
    }
    pub fn get_access_key(&self) -> &Option<String>{
        &self.access_key
    }
    pub fn get_database(&self) -> &String{
        &self.database
    }


}

