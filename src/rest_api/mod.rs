use lazy_static::lazy_static;
use reqwest::{
    blocking::{self},
    header::{self, HeaderMap, HeaderValue},
};

lazy_static! {
    static ref ENDPOINT: String = {
        let mut args = std::env::args();
        args.next();
        args.next()
            .unwrap_or_else(|| "http://localhost:8055".to_owned())
    };
    static ref HTTP_CLIENT: blocking::Client = {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer admin_token"),
        );
        blocking::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    };
}

pub trait SaveToDb {
    fn save_to_db(&self) -> anyhow::Result<()>;
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
mod network;
mod rule;
mod screenshot;

pub use event::*;
pub use network::*;
pub use rule::*;
pub use screenshot::*;
