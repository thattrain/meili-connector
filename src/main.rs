use std::ops::Deref;
use std::time::Duration;
use futures::task::SpawnExt;

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

// #[tokio::main]
// async  fn main(){
//     println!("Start program !!!!");
//     let handle = tokio::spawn(async {
//         for i in vec![1, 2, 3, 4, 5] {
//             println!("i: {}", i);
//             tokio::time::sleep(Duration::from_secs(2)).await;
//         }
//         println!("Done spawned task");
//     });
//
//     // tokio::time::sleep(Duration::from_secs(5)).await;
//     println!("Expected this line to print before spawn task finish execute");
//
// }
