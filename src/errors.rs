use reqwest;
use serde_json;
use tantivy;
use std::io;

error_chain!{

    foreign_links {
        Serde(serde_json::Error);
        Reqwest(reqwest::Error);
        Io(io::Error);
        Tantivy(tantivy::Error);
    }
}
