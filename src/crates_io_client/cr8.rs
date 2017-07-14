#[allow(unused_imports)]
use std::io::Read;
#[allow(unused_imports)]
use serde_json::Value;
use reqwest;

use std::collections::HashMap;

use errors::*;

const CRATES_IO_URL: &'static str = "https://crates.io";

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    #[serde(rename = "crate")]
    pub crate_: String,
    pub created_at: String,
    pub dl_path: String,
    pub downloads: i32,
    pub features: HashMap<String, Vec<String>>,
    pub id: i32,
    pub links: HashMap<String, String>,
    pub num: String,
    pub updated_at: String,
    pub yanked: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Keyword {
    pub crates_cnt: i32,
    pub created_at: String,
    pub id: String,
    pub keyword: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    pub category: String,
    pub crates_cnt: i32,
    created_at: String,
    description: String,
    id: String,
    slug: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Badge {
    pub attributes: Attrs,
    pub badge_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attrs {
    pub branch: Option<String>,
    pub repository: String,
    pub service: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateDetail {
    pub badges: Vec<Badge>,
    pub categories: Option<Vec<String>>,
    pub created_at: String,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub downloads: i32,
    pub exact_match: bool,
    pub homepage: Option<String>,
    pub id: String,
    pub keywords: Vec<String>,
    pub license: Option<String>,
    pub links: HashMap<String, Option<String>>,
    pub max_version: String,
    pub name: String,
    pub repository: Option<String>,
    pub updated_at: String,
    pub versions: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Crate {
    pub categories: Option<Vec<Category>>,
    #[serde(rename = "crate")]
    pub crate_: CrateDetail,
    pub keywords: Vec<Keyword>,
    pub versions: Vec<Version>,
}

impl Crate {
    pub fn get(name: &str) -> Result<Crate> {
        let mut r = reqwest::get(&format!("{}/api/v1/crates/{}", CRATES_IO_URL, name))?;
        if r.status().is_success() {
            Ok(r.json()?)
        } else {
            Err(ErrorKind::Msg("Not Found".into()).into())
        }
    }
}


