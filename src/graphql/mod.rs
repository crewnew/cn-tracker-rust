#![allow(non_camel_case_types)]

use reqwest::{
    blocking,
    header::{HeaderMap, HeaderValue},
};

lazy_static! {
    static ref USER_ID: String = "6db77da0-ea5d-4dac-899c-aa371ecc0c51".to_owned();
    static ref HASURA_ADMIN_SECRET: String =
        std::env::var("HASURA_ADMIN_SECRET").unwrap_or_else(|_| "myadminsecret".to_owned());
    static ref ENDPOINT: String = {
        let mut args = std::env::args();
        args.next();
        args.next()
            .unwrap_or_else(|| "https://api.klarity.app/v1/graphql".to_owned())
    };
    static ref HTTP_CLIENT: blocking::Client = {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Hasura-Admin-Secret",
            HeaderValue::from_static(HASURA_ADMIN_SECRET.as_str()),
        );
        headers.insert(
            "X-Hasura-User-Id",
            HeaderValue::from_static(USER_ID.as_str()),
        );
        blocking::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    };
}

pub(self) use graphql_client::{GraphQLQuery, Response};

type Uuid = String;
type Citext = String;
type Bigint = i64;
type Float8 = f32;
type Timestamptz = String;
type Bytea = Vec<u8>;
type Jsonb = Vec<u8>;

mod project_rules;
mod user_events;

pub use project_rules::*;
pub use user_events::*;
