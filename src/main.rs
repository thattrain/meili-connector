use std::ops::Deref;

use sqlx::{Connection, Executor};

use crate::config::Config;
use crate::synchronizer::Synchronizer;

mod meili;
mod config;
mod data_source;
mod synchronizer;

#[tokio::main]
async  fn main(){
    let config = Config::read_config("config.yaml".to_string());

    let synchronizer = Synchronizer::new(config.meilisearch, config.data_source, config.synchronize_tables);
    synchronizer.sync().await;


}
