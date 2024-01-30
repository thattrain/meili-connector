use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SupportedDataSource{
    PostgresSQL,
    // MongoDB,
    // MySQL
}