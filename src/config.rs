
use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use crate::meili::meili_config::MeiliConfig;
use crate::data_source::data_source_setting::DataSourceConfig;
use crate::meili::index_setting::IndexSetting;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config{
    pub meilisearch: MeiliConfig,
    pub data_source: DataSourceConfig,
    pub synchronize_tables: Vec<IndexSetting>
}

impl Config{
    pub fn read_config(file_name: String) -> Self{
        let file = std::fs::File::open(file_name).expect("Can not open config file");
        let config: Config = serde_yaml::from_reader(file).expect("Could not read value from config file");
        return config;
    }

    //todo: impl validate config and error handling here

}


