#![cfg(feature = "graphql")]
use gql_client::Client;
use lazy_static::lazy_static;

const ENDPOINT: &str = "http://localhost:8055/graphql";

lazy_static! {
    static ref CLIENT: Client<'static> = Client::new_with_headers(
        ENDPOINT,
        std::collections::HashMap::from([("Authorization", "Bearer admin_key")])
    );
}

#[async_trait]
trait SaveToDb {
    async fn save_to_db(&self) -> anyhow::Result<()>;
}

#[derive(Serialize)]
struct Variables<'a> {
    data: &'a (dyn erased_serde::Serialize + Send + Sync),
}

#[derive(Debug, Clone, Deserialize)]
struct Id {
    id: String,
}

mod event;
mod rule;

pub use event::*;
pub use rule::*;
