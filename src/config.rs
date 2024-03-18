use std::fs::File;
use std::io::Read;
use owo_colors::{DynColors, OwoColorize};
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
    pub fn load_banner(file_name: &str){
        let mut file = File::open(file_name).expect("Can not load banner from file");
        let mut banner = String::new();
        file.read_to_string(&mut banner).unwrap();

        println!("{}\n\n\n", banner.truecolor(255,0,255). bold());
    }

    pub fn read_config(file_name: String) -> Self{
        let file = File::open(file_name).expect("Can not open config file");
        let config: Config = serde_yaml::from_reader(file).expect("Could not read value from config file");
        return config;
    }

    //todo: impl validate config and error handling here

}


