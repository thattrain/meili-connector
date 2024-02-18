use meilisearch_sdk::Client;
pub mod meili_config;
pub mod index_setting;
pub mod meili_enum;

//todo: implement interaction with meilisearch instance
pub struct MeiliSearchService{
     meili_client:  Client,
}

impl MeiliSearchService{
    // pub fn new(meili_config: MeiliConfig) -> Self{
    //     let mut client = Client::new(meili_config.get_api_url(), meili_config.get_admin_api_key());
    //      MeiliSearchService {
    //         meili_client: *client
    //     }
    // }
    //
    // pub fn get_meili_client(self) -> *mut Client{
    //     self.meili_client
    // }

    // pub fn add_documents(self, index_name: String) {
    //     let index = &self.get_meili_client().index(index_name);
    //     let task = index.add_documents_ndjson()
    //
    // }
}
