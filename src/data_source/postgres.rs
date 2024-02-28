use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use sqlx::{Executor, PgPool};
use tokio::task;

use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;
use crate::data_source::deserializer::SerMapPgRow;
use crate::meili::index_setting::IndexSetting;

type JsonValues = Arc<Mutex<Vec<Value>>>;

pub struct PostgresSource {
    index_setting: Arc<IndexSetting>,
    data_source_config: DataSourceConfig
}

impl PostgresSource{

    fn get_index_setting(&self) -> &IndexSetting{
        return &self.index_setting;
    }

    pub fn new(index_setting: Arc<IndexSetting>, data_source_config: DataSourceConfig) -> PostgresSource{
        PostgresSource{
            index_setting,
            data_source_config
        }
    }

    async fn create_connection(&self) -> PgPool {
        let data_source_config = &self.data_source_config;
        let db_url = format!("postgresql://{}:{}@{}:{}/{}",
                                 data_source_config.get_username(),
                                 data_source_config.get_password(),
                                 data_source_config.get_host(),
                                 data_source_config.get_port(),
                                 data_source_config.get_database());

        let pg_pool = PgPool::connect(&db_url)
            .await
            .expect(format!("Can not connect to Postgres instance at {}:{}", data_source_config.get_username(), data_source_config.get_port()).as_str());

        pg_pool
    }

}

impl DataSource for PostgresSource {
    async fn get_total_record_num(&self) -> i64 {
        let pg_pool = self.create_connection().await;
        let table_name = self.get_index_setting().get_index_name();
        let query= format!("SELECT COUNT (*) FROM {}", table_name);
        let count: i64 = sqlx::query_scalar(query.as_str()).fetch_one(&pg_pool).await.expect("Failed to get total record !!!!!");
        return count;
    }

     async fn get_full_data(&self, size: i64) -> Vec<Value>{
        let total_records = self.get_total_record_num().await;
        println!("Total record need to be sync: {} from table: {}", total_records, &self.index_setting.get_index_name());

        let mut offset: i64 = 0;
        let mut current_page = 0;
        let total_page = ((total_records % size ) as f64).round();
        //todo: should we use tokio::stream instead ?????
        let pg_total_records: JsonValues = Arc::new(Mutex::new(Vec::new()));

         let sync_fields = self.get_index_setting().get_sync_fields().as_ref().unwrap();
         let mut fields = String::new();
         if sync_fields.len() > 0 {
             fields.push_str(sync_fields.join(", ").as_str());
         }else {
             fields.push_str("*");
         }

        while current_page <= total_page as i64 {
            let pg_pool = self.create_connection().await;
            let index_setting = self.index_setting.clone();
            let query_fields = fields.clone();

            //spawn a new task to query by limit and offset
            let mut query_result = tokio::spawn(async move {
                let query= format!("SELECT {} FROM {} ORDER BY {} LIMIT {} OFFSET {}",
                                   query_fields,
                                   &index_setting.get_index_name(),
                                   &index_setting.get_primary_key(),
                                   size, offset
                );

                let rows = pg_pool.fetch_all(query.as_str()).await.unwrap();
                let mut result = Vec::new();
                for row in rows{
                    let row = SerMapPgRow::from(row);
                    let json_row: Value = serde_json::to_value(&row).unwrap();
                    result.push(json_row);
                }
                result
            }).await.expect("Error when query full data from Postgres database");

            pg_total_records.lock().unwrap().append(&mut query_result);
            offset += size;
            current_page += 1;
        }

        return pg_total_records.lock().unwrap().to_vec();
    }
}