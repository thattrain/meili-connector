use crate::config::Config;
use crate::data_source::DataSource;
use crate::synchronizer::Synchronizer;

mod meili;
mod config;
mod data_source;
mod synchronizer;

#[tokio::main]
async  fn main(){
    //todo: handle command line options
    Config::load_banner("banner.txt");
    let config = Config::read_config("examples/postgres/postgres.yaml".to_string());
    let source_config = config.data_source.clone();
    let synchronize_tables = config.synchronize_tables.clone();
    let synchronizer = Synchronizer::get_synchronizer(config.meilisearch, source_config.clone(), synchronize_tables);
    synchronizer.sync().await;


}
