use serde_json::{to_string, Value};
use serde_json::map::Values;
use sqlx::{Column, ColumnIndex, PgPool, Row};
use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;
use crate::meili::index_setting::IndexSetting;



pub struct PostgresSource<'a> {
    synchronize_tables: &'a Vec<IndexSetting>,
    data_source_config: &'a DataSourceConfig
}

impl PostgresSource <'_>{

    pub fn new<'a>(synchronize_tables: &'a Vec<IndexSetting>, data_source_config: &'a DataSourceConfig) -> PostgresSource<'a>{
        PostgresSource{
            synchronize_tables,
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

impl <'a> DataSource for PostgresSource <'a> {
     async fn get_full_data(&self) -> String{
        let pg_pool = self.create_connection().await;
        let query = sqlx::query("SELECT * FROM users");
        let result = query.fetch_all(&pg_pool).await;



        return "".to_string();
    }
}