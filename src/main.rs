use clap::Parser;
use env_logger::Builder;
use log::{info, LevelFilter};

use crate::config::{CLI, Commands, Config};
use crate::synchronizer::Synchronizer;

mod meili;
mod config;
mod data_source;
mod synchronizer;

#[tokio::main]
async  fn main(){
    Config::load_banner("banner.txt");
    let args = CLI::parse();
    let config_file = args.config;
    init_logger(args.debug);

    let config = Config::read_config(config_file.to_str().unwrap());
    let Config {data_source, synchronize_tables, meilisearch} = config;
    let synchronizer = Synchronizer::get_synchronizer(meilisearch, data_source, synchronize_tables);

    match args.cmd {
        Commands::Sync => {
            synchronizer.sync().await;
        }
        Commands::Refresh => {
            info!("TODO: impl later")
            //todo: impl refresh data by swap index
        }
    }

}

fn init_logger(is_debug: bool){
    let log_level = if is_debug {
        LevelFilter::Debug
    }else {
        LevelFilter::Info
    };

    Builder::new()
        .filter(None, log_level)
        .init();
}