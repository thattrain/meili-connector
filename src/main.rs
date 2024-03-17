use crate::config::Config;
use crate::synchronizer::Synchronizer;

mod meili;
mod config;
mod data_source;
mod synchronizer;

#[tokio::main]
async  fn main(){
    //todo: print banner and handle options from command line here
    let config = Config::read_config("config.yaml".to_string());
    let synchronizer = Synchronizer::get_synchronizer(config.meilisearch, config.data_source, config.synchronize_tables);
    synchronizer.sync().await;

}
