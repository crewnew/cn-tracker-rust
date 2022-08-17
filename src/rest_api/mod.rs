use reqwest::{
    blocking::{self},
    header::{HeaderMap, HeaderValue},
};

lazy_static! {
    static ref ENDPOINT: String = {
        let mut args = std::env::args();
        args.next();
        args.next()
            .unwrap_or_else(|| "http://localhost:8080/api/rest".to_owned())
    };
    static ref HTTP_CLIENT: blocking::Client = {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Hasura-Admin-Secret",
            HeaderValue::from_static("hello123"),
        );
        headers.insert(
            "X-Hasura-User-Id",
            HeaderValue::from_static("9c68f86f-c5a5-4b3e-a317-466c6bafcc42"),
        );
        headers.insert("X-Hasura-Role", HeaderValue::from_static("user"));
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

mod event;
mod rule;
mod screenshot;

pub use event::*;
pub use rule::*;
pub use screenshot::*;
