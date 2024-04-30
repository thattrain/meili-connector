use clap::Parser;
use log::LevelFilter;

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

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(synchronize_tables.len())
        .enable_all()
        .build()
        .unwrap();

    let synchronizer = Synchronizer::get_synchronizer(meilisearch, data_source, synchronize_tables, runtime);

    match args.cmd {
        Commands::Sync => {
            synchronizer.sync().await;
        }
        Commands::Refresh => {
            synchronizer.refresh().await;
        }
        Commands::Status => {
            synchronizer.status().await;
        }
    }
}

fn init_logger(is_debug: bool){
    let log_level = if is_debug {
        LevelFilter::Debug
    }else {
        LevelFilter::Info
    };

    env_logger::Builder::new()
        .filter(None, log_level)
        .init();
}

