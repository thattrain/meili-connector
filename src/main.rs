use crate::config::Config;
use crate::synchronizer::Synchronizer;

mod meili;
mod config;
mod data_source;
mod synchronizer;

#[tokio::main]
async  fn main(){
    //todo: handle command line options
    Config::load_banner("banner.txt");
    let config = Config::read_config("examples/postgres/postgres.yaml");
    let Config {data_source, synchronize_tables, meilisearch} = config;
    let synchronizer = Synchronizer::get_synchronizer(meilisearch, data_source, synchronize_tables);
    synchronizer.sync().await;


}
