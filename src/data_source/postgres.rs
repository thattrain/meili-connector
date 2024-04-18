use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::Poll,
    time::{SystemTime, UNIX_EPOCH}
};

use async_trait::async_trait;
use bytes::Bytes;
use futures::{future::{self}, ready, Sink, StreamExt};
use regex::Regex;
use serde_json::{json, Map, Value};
use sqlx::{Executor, PgPool};
use tokio::sync::mpsc::Sender;
use tokio_postgres::{CopyBothDuplex, NoTls, SimpleQueryMessage, types::PgLsn};

use crate::data_source::data_source_setting::DataSourceConfig;
use crate::data_source::DataSource;
use crate::data_source::deserializer::SerMapPgRow;
use crate::meili::EventMessage;
use crate::meili::index_setting::IndexSetting;
use crate::meili::Event::*;

type JsonValues = Arc<Mutex<Vec<Value>>>;

const SECONDS_FROM_UNIX_EPOCH_TO_2000: u128 = 946684800;
const SLOT_PREFIX: &str = "meili_";

pub struct Postgres {
    index_setting: IndexSetting,
    data_source_config: DataSourceConfig,
}
struct Publication {
    client: Arc<tokio_postgres::Client>,
    schema_name: String,
    table_name: String,
    publish_cols: Vec<String>,
}
struct Slot {
    client: Arc<tokio_postgres::Client>,
    name: String,
    lsn: Option<PgLsn>,
}
struct DBClient {
    client: tokio_postgres::Client,
}
struct ReplicationEventNotifier {
    commit_lsn: PgLsn,
    slot_name: String,
    schema_name: String,
    table_name: String,
    registered_cols: Vec<String>,
    client: Arc<tokio_postgres::Client>,
    stream: Option<Pin<Box<CopyBothDuplex<Bytes>>>>,
}

impl Postgres {

    fn get_index_setting(&self) -> &IndexSetting{
        return &self.index_setting;
    }

    pub fn new(index_setting: IndexSetting, data_source_config: DataSourceConfig) -> Self {
        Self{
            index_setting,
            data_source_config,
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

impl Publication {
    pub fn new(
        client: Arc<tokio_postgres::Client>,
        schema_name: &String,
        table_name: &String,
        publish_cols: Vec<String>
    ) -> Self {
        Self {
            client,
            schema_name: schema_name.clone(),
            table_name: table_name.clone(),
            publish_cols,
        }
    }

    #[inline]
    pub fn pub_name(&self) -> String {
        format!("{}{}",SLOT_PREFIX , self.table_name)
    }

    pub async fn check_exists(&self) -> Result<bool, tokio_postgres::Error> {
        let pub_name = self.pub_name();
        let query = format!(
            "SELECT schemaname, tablename
                FROM pg_publication p
                JOIN pg_publication_tables pt ON p.pubname = pt.pubname
                WHERE p.pubname = '{}'",
            pub_name
        );
        let result = self.client.simple_query(&query).await?;
        let rows = result
            .into_iter()
            .filter_map(|msg| match msg {
                SimpleQueryMessage::Row(row) => Some(row),
                _ => None,
            })
            .collect::<Vec<_>>();

        return if let Some(publication) = rows.first() {
            let schema_name = publication.get("schemaname").unwrap().to_string();
            let table_name = publication.get("tablename").unwrap().to_string();
            println!(
                "Found publication {:?}/{:?}, ready to start replication",
                schema_name, table_name
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn create(&self, pg_version: f32) -> Result<u64, tokio_postgres::Error> {

        //from postgres version 15 and above, we can config which fields to publish
        let query = if pg_version > 15.0 && self.publish_cols.len() > 0 {
            let mut registered_fields = String::new();
            registered_fields.push_str(&self.publish_cols.join(", ").as_str());
            format!(
                "CREATE PUBLICATION \"{}\" FOR TABLE {} ({})",
                self.pub_name(),
                self.table_name,
                registered_fields
            )
        }else {
            format!(
                "CREATE PUBLICATION \"{}\" FOR TABLE {}",
                self.pub_name(),
                self.table_name,
            )
        };

        println!("Creating publication: {:?}", query);
        let result = self.client.execute(&query, &[]).await?;
        println!("Created publication: {:?}", result);
        Ok(result)
    }
}

impl Slot {
    pub fn new(client: Arc<tokio_postgres::Client>, slot_name: &String) -> Self {
        Self {
            client,
            name: slot_name.clone(),
            lsn: None,
        }
    }

    pub async fn get_confirmed_lsn(&mut self) -> Result<(), tokio_postgres::Error> {
        let query = format!(
            "SELECT confirmed_flush_lsn FROM pg_replication_slots WHERE slot_name = '{}'",
            self.name
        );
        let result = self.client.simple_query(&query).await?;
        let rows = result
            .into_iter()
            .filter_map(|msg| match msg {
                SimpleQueryMessage::Row(row) => Some(row),
                _ => None,
            })
            .collect::<Vec<_>>();

        if let Some(slot) = rows.first() {
            let lsn = slot.get("confirmed_flush_lsn").unwrap().to_string();
            self.lsn = Some(lsn.parse::<PgLsn>().unwrap());
        }

        Ok(())
    }

    pub async fn create(&mut self) -> Result<(), tokio_postgres::Error> {

        //use wal2json plugin to create a logical replication slot
        let slot_query = format!(
            "CREATE_REPLICATION_SLOT {} LOGICAL \"wal2json\" NOEXPORT_SNAPSHOT",
            self.name
        );
        let result = self.client.simple_query(&slot_query).await?;

        let lsn = result
            .into_iter()
            .filter_map(|msg| match msg {
                SimpleQueryMessage::Row(row) => Some(row),
                _ => None,
            })
            .collect::<Vec<_>>()
            .first()
            .unwrap()
            .get("consistent_point")
            .unwrap()
            .to_owned();
        println!("Created replication slot: {:?}", lsn);
        self.lsn = Some(lsn.parse::<PgLsn>().unwrap());
        Ok(())
    }
}

impl DBClient {
    pub async fn new(db_config: &str) -> Result<Self, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(db_config, NoTls).await?;
        tokio::spawn(async move { connection.await });
        Ok(Self { client })
    }
}

impl ReplicationEventNotifier{

    fn new(
        client: Arc<tokio_postgres::Client>,
        slot: Slot,
        publication: Publication,
    ) -> Self{

        Self {
            schema_name: publication.schema_name.clone(),
            table_name: publication.table_name.clone(),
            registered_cols: publication.publish_cols.clone(),
            // lsn must be assigned at this point else we panic
            commit_lsn: slot.lsn.unwrap().clone(),
            slot_name: slot.name.clone(),
            client,
            stream: None,
        }
    }

    async fn start_listening(&mut self, event_sender: Sender<EventMessage>){
        let full_table_name = format!("{}.{}", self.schema_name, self.table_name);
        let wal2json_options = vec![
            ("pretty-print", "false"),
            ("include-transaction", "true"),
            ("include-lsn", "true"),
            ("include-timestamp", "true"),
            ("include-pk", "true"),
            ("format-version", "2"),
            ("include-xids", "true"),
            ("add-tables", &full_table_name),
        ];
        let start_lsn = self.commit_lsn.to_string();
        let query = format!(
            "START_REPLICATION SLOT {} LOGICAL {} ({})",
            self.slot_name,
            start_lsn,
            // specify table for replication
            wal2json_options
                .iter()
                .map(|(k, v)| format!("\"{}\" '{}'", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let duplex_stream = self
            .client
            .copy_both_simple::<Bytes>(&query)
            .await
            .unwrap();

        // Pin the stream
        self.stream = Some(Box::pin(duplex_stream));

        // listen for changes
        loop {
            match self.stream.as_mut().unwrap().next().await {
                Some(Ok(event)) => {
                    // (todo:) should return error?
                    self.process_wal2json_event(&event, event_sender.clone()).await;
                }
                Some(Err(e)) => {
                    //todo: panic or try to recover?
                    panic!("Error reading from stream: {}", e);
                }
                None => {
                    //todo: panic here??
                    println!("Stream closed");
                    break;
                }
            }
        }
    }

    // this function process WAL event decoded by wal2json plugin
    async fn process_wal2json_event(&mut self, event: &[u8], event_sender: Sender<EventMessage>) {
        let identify_byte = event[0];
        match identify_byte {        // first byte is identified byte
            b'w' => {   // byte indicate message as WAL data
                // first 24 bytes are metadata
                let json: Value = serde_json::from_slice(&event[25..]).unwrap();
                // handle WAL data stream
                self.process_change_event(json, event_sender).await;
            }
            b'k' => {   // byte indicate primary keepalive message
                let last_byte = event.last().unwrap();
                //if last_byte == 1 then reply this message immediately to void timeout disconnect
                if last_byte == &1 {
                    println!("Send keep alive message - @LSN:{:x?}", self.commit_lsn);
                    let buf = prepare_standby_status_update(self.commit_lsn);
                    self.send_standby_status_update(buf).await;
                }
            }
            _ => (),
        }
    }

    async fn process_change_event(&mut self, record: Value, event_sender: Sender<EventMessage>) {
        match record["action"].as_str().unwrap() {
            "B" => {
                // println!("Begin===");
                // println!("{}", serde_json::to_string_pretty(&record).unwrap());
                let lsn_str = record["nextlsn"].as_str().unwrap();
                self.commit_lsn = lsn_str.parse::<PgLsn>().unwrap();
            }
            "C" => {
                let end_lsn_str = record["nextlsn"].as_str().unwrap();
                let end_lsn = end_lsn_str.parse::<PgLsn>().unwrap();
                if end_lsn != self.commit_lsn {
                    println!(
                        "commit and begin next_lsn don't match: {:?}",
                        record["nextlsn"]
                    );
                }
                // println!("Commit===");
                // println!("{}", serde_json::to_string_pretty(&record).unwrap());
                self.commit().await;
            }
            "I" => {        //insert event
                let columns =  record["columns"].as_array().unwrap();
                let mut map = Map::new();
                for column in columns.iter(){
                    let column_name = column.get("name").unwrap().as_str().unwrap();
                    let column_value = column.get("value").unwrap().to_string();
                    map.insert(column_name.parse().unwrap(), column_value.parse().unwrap());
                }
                let json_obj =  Value::Object(map);
                let table_name = record["table"].as_str().unwrap();
                let insert_message = EventMessage{
                    event_type: Insert,
                    index_name: table_name.to_string(),
                    payload: Arc::new(vec![json_obj])
                };
                //todo: handle error???
                let _ = &event_sender.send(insert_message).await.unwrap();
            }
            "U" => {        //update event
                let columns =  record["columns"].as_array().unwrap();
                let mut map = Map::new();
                for column in columns.iter(){
                    let column_name = column.get("name").unwrap().as_str().unwrap();
                    let column_value = column.get("value").unwrap().to_string();
                    println!("Col name: {} - value: {}", column_name, column_value.as_str());
                    if self.registered_cols.iter().any(|col_name| col_name == column_name){
                        map.insert(column_name.parse().unwrap(), column_value.parse().unwrap());
                    }else {
                        //ignore event from unregistered fields
                        return;
                    }
                }
                let json_obj =  Value::Object(map);
                let table_name = record["table"].as_str().unwrap();
                let insert_message = EventMessage{
                    event_type: Update,
                    index_name: table_name.to_string(),
                    payload: Arc::new(vec![json_obj])
                };
                //todo: handle error???
                let _ = &event_sender.send(insert_message).await.unwrap();
            }
            "D" => {        //delete event
                let pk_key = record["identity"].get(0).unwrap().get("name").unwrap().as_str().unwrap();
                let value = record["identity"].get(0).unwrap().get("value");
                let table_name = record["table"].as_str().unwrap();
                let deleted_record = json!({ pk_key: value });
                let delete_message = EventMessage{
                    event_type: Delete,
                    index_name: table_name.to_string(),
                    payload: Arc::new(vec![deleted_record])
                };
                //todo: handle error???
                let _ = &event_sender.send(delete_message).await.unwrap();
            }
            _ => {
                println!("unknown event: {}",  record["action"].as_str().unwrap());
            }
        }
    }

    async fn commit(&mut self) {
        let buf = prepare_standby_status_update(self.commit_lsn);
        self.send_standby_status_update(buf).await;
    }

    async fn send_standby_status_update(&mut self, buf: Bytes) {
        println!("Trying to send SSU");
        let mut next_step = 1;
        future::poll_fn(|cx| loop {
            // println!("Doing step:{}", next_step);
            match next_step {
                1 => {
                    ready!(self.stream.as_mut().unwrap().as_mut().poll_ready(cx)).unwrap();
                }
                2 => {
                    self.stream
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .start_send(buf.clone())
                        .unwrap();
                }
                3 => {
                    ready!(self.stream.as_mut().unwrap().as_mut().poll_flush(cx)).unwrap();
                }
                4 => return Poll::Ready(()),
                _ => panic!(),
            }
            next_step += 1;
        })
            .await;
        println!("Sent SSU");
    }


}

fn prepare_standby_status_update(write_lsn: PgLsn) -> Bytes {
    let write_lsn_bytes = u64::from(write_lsn).to_be_bytes();
    let time_since_2000: u64 = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
        - (SECONDS_FROM_UNIX_EPOCH_TO_2000 * 1000 * 1000))
        .try_into()
        .unwrap();

    // see here for format details: https://www.postgresql.org/docs/10/protocol-replication.html
    let mut data_to_send: Vec<u8> = vec![];
    // Byte1('r'); Identifies the message as a receiver status update.
    data_to_send.extend_from_slice(&[114]); // "r" in ascii

    // The location of the last WAL byte + 1 received and written to disk in the standby.
    data_to_send.extend_from_slice(write_lsn_bytes.as_ref());

    // The location of the last WAL byte + 1 flushed to disk in the standby.
    data_to_send.extend_from_slice(write_lsn_bytes.as_ref());

    // The location of the last WAL byte + 1 applied in the standby.
    data_to_send.extend_from_slice(write_lsn_bytes.as_ref());

    // The client's system clock at the time of transmission, as microseconds since midnight on 2000-01-01.
    //0, 0, 0, 0, 0, 0, 0, 0,
    data_to_send.extend_from_slice(&time_since_2000.to_be_bytes());
    // Byte1; If 1, the client requests the server to reply to this message immediately. This can be used to ping the server, to test if the connection is still healthy.
    data_to_send.extend_from_slice(&[1]);

    Bytes::from(data_to_send)
}
#[async_trait]
impl DataSource for Postgres {
    async fn version(&self) -> f32 {
        let pg_pool = self.create_connection().await;
        let pg_version_str: String = sqlx::query_scalar("SHOW server_version").fetch_one(&pg_pool).await.expect("Can not get Postgres SQL version");
        let re = Regex::new(r"(?P<version>\d+\.\d+)").unwrap();
        if let Some(captures) = re.captures(pg_version_str.as_str()) {
            if let Some(version) = captures.name("version") {
                let version_value = version.as_str().parse::<f32>().unwrap();
                version_value
            }else {
                0.0
            }
        }else {
            0.0
        }
    }

    async fn total_record(&self) -> i64 {
        let pg_pool = self.create_connection().await;
        let table_name = self.get_index_setting().get_index_name();
        let query= format!("SELECT COUNT (*) FROM {}", table_name);
        let count: i64 = sqlx::query_scalar(query.as_str()).fetch_one(&pg_pool).await.expect("Failed to get total record !!!!!");
        return count;
    }

     async fn get_data(&self, size: i64, offset: i64) -> Vec<Value>{
         let sync_fields = self.get_index_setting().get_sync_fields().as_ref().unwrap();
         let mut fields = String::new();
         if sync_fields.len() > 0 {
             fields.push_str(sync_fields.join(", ").as_str());
         }else {
             fields.push_str("*");
         }

        let pg_pool = self.create_connection().await;
        let mut result = Vec::new();
        let query= format!("SELECT {} FROM \"{}\" ORDER BY {} LIMIT {} OFFSET {}",
                           fields,
                           &self.index_setting.get_index_name(),
                           &self.index_setting.get_primary_key(),
                           size, offset
        );

        let rows = pg_pool.fetch_all(query.as_str()).await.expect("Can not query Postgres !!!");
        for row in rows{
            let row = SerMapPgRow::from(row);
            let json_row: Value = serde_json::to_value(&row).unwrap();
            result.push(json_row);
        }
         return result;
    }

    async fn start_event_notifier(&self, index_setting: &IndexSetting, data_source_config: &DataSourceConfig, event_sender: Sender<EventMessage>){

        let db_string = format!("user={} password={} host={} port={} dbname={}",
                                data_source_config.get_username(),
                                data_source_config.get_password(),
                                data_source_config.get_host(),
                                data_source_config.get_port(),
                                data_source_config.get_database()
        );

        //create a Postgres publication if not exist
        let client = Arc::new(DBClient::new(db_string.as_str()).await.unwrap().client);
        let schema_name = "public".to_string();
         let publish_cols: Vec<String> = index_setting.get_sync_fields()
             .as_ref()
             .unwrap()
             .iter()
             .map(|col_name| col_name.to_string())
             .collect::<Vec<_>>();

        let mut publication = Publication::new(
            Arc::clone(&client),
            &schema_name,
            index_setting.get_index_name(),
            publish_cols
        );

        if !publication.check_exists().await.unwrap() {
            let pg_version = self.version().await;
            publication.create(pg_version).await.unwrap();
        }

        //create a slot if not exist subscribe to the publication
        let slot_name = format!("{}{}", SLOT_PREFIX, index_setting.get_index_name());
        let repl_client = Arc::new(
            DBClient::new(&format!("{} replication=database", db_string))
                .await
                .unwrap()
                .client,
        );
        let mut slot = Slot::new(Arc::clone(&repl_client), &slot_name);
        slot.get_confirmed_lsn().await.unwrap();
        if slot.lsn.is_none() {
            println!("Replication slot {slot_name} does not exist - Creating replication slot: {slot_name}");
            slot.create().await.unwrap();
        }

        let mut event_notifier = ReplicationEventNotifier::new(
            Arc::clone(&repl_client),
            slot,
            publication
        );

         println!("Start event listener for table: {}", index_setting.get_index_name());
         event_notifier.start_listening(event_sender).await;

    }

}