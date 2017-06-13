use reqwest;
use serde_json;

error_chain!{
    foreign_links {
        Serde(serde_json::Error);
        Reqwest(reqwest::Error);
    }
}
