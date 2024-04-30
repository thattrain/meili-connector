use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use log::{info};
use owo_colors::{OwoColorize};
use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use crate::meili::meili_config::MeiliConfig;
use crate::data_source::data_source_setting::DataSourceConfig;
use crate::meili::index_setting::IndexSetting;


//region handle cli options
/// Meili-connector synchronize your datasource with Meilisearch instance base on your needs.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CLI{
    /// .yaml file to declare synchronize configurations
    #[clap(short, long, value_name = "configuration file")]
    pub config: PathBuf,

    /// Show debugging information
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    #[clap(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands{
    /// Sync data source with Meilisearch instance
    Sync,

    /// Refresh data by swap index
    Refresh,

    /// Check whether data in datasource is consistent with data in Meilisearch
    Status
}
//endregion handle cli options


// region parse config
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

    pub fn read_config(file_name: &str) -> Self{
        let file = File::open(file_name).expect("Can not open config file");
        info!("Load config file at: {}", file_name);
        let config = serde_yaml::from_reader(file);
        match config {
            Ok(config) => return config,
            Err(error) => panic!("Error while parsing config: {:?}", error)
        }
    }

    //todo: impl validate config and error handling here

}

//endregion parse config


