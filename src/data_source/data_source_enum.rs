use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SupportedDataSource{
    PostgresSQL,
    MongoDB,
    MySQL
}