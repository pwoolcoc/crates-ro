extern crate serde_json;
extern crate reqwest;
extern crate walkdir;
extern crate tantivy;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate tera;

pub mod errors;
pub mod index;
pub mod page_generator;
pub mod crates_io_client;

