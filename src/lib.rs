#![warn(clippy::print_stdout)]
#[macro_use]
extern crate serde;
#[macro_use]
extern crate log;
#[macro_use]
extern crate enum_dispatch;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate graphql_client;

pub mod capture;
pub mod graphql;
pub mod rest_api;
pub mod scripting;
pub mod util;
