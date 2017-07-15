#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate collection_macros;
extern crate crates_no;
extern crate clap;
extern crate tantivy;
extern crate rocket;
extern crate rocket_contrib;
extern crate itertools;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};

use itertools::Itertools;
use rocket::State;
use rocket::response::NamedFile;
use rocket_contrib::Template;
use clap::{App, Arg};

use crates_no::page_generator::directory_for_name;
use crates_no::index::Indexer;

pub struct Config {
    pub indiv_crate_dir: PathBuf,
    pub letters_dir: PathBuf,
}

#[get("/")]
fn index() -> Template {
    let h: HashMap<&'static str, &'static str> = HashMap::new();
    Template::render("index", &h)
}

#[get("/search?<query>")]
fn search(query: Query, index: State<Arc<Mutex<Indexer>>>) -> Template {
    println!("Searching for '{}'", query.q.trim());
    let index = index.clone();
    let index = index.lock().unwrap();
    let results = index.search(&query.q.trim()).unwrap();
    let results = results.into_iter().map(|s| {
            SearchResult::from(s)
    })
    .unique_by(|s| s.name.clone())
    .sorted_by(|s, t| Ord::cmp(&s.name, &t.name));

    let m = hashmap! {
        "results" => results,
    };
    Template::render("search", &m)
}

#[get("/crates?<letter>")]
fn letter(letter: Letter, config: State<Config>) -> Option<NamedFile> {
    let fname = config.letters_dir.join(letter.letter);
    NamedFile::open(fname).ok()
}

#[get("/crates/<cr8>")]
fn cr8(cr8: String, config: State<Config>) -> Option<NamedFile> {
    let crate_loc = directory_for_name(&cr8);
    let directory = config.indiv_crate_dir.join(&crate_loc);
    let fname = directory.join(Path::new(&cr8));
    NamedFile::open(fname).ok()
}

#[derive(Debug, Serialize)]
struct SearchResult {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

impl From<(String, String, Option<String>)> for SearchResult {
    fn from(result: (String, String, Option<String>)) -> SearchResult {
        SearchResult {
            name: result.0,
            version: result.1,
            description: result.2,
        }
    }
}

#[derive(FromForm)]
struct Query {
    pub q: String,
}

#[derive(FromForm)]
struct Letter {
    pub letter: String,
}

fn main() {
    let matches = App::new("crates-no")
                        .arg(Arg::with_name("INDEX")
                                 .required(true)
                                 .help("Location of search index")
                                 .index(1))
                        .arg(Arg::with_name("pages-dir")
                                 .long("pages-dir")
                                 .takes_value(true)
                                 .value_name("pages-dir"))
                        .arg(Arg::with_name("static-dir")
                                 .long("static-dir")
                                 .takes_value(true)
                                 .value_name("static-dir"))
                     .get_matches();
    let index_dir = matches.value_of("INDEX").unwrap();
    let indexer = Indexer::searcher(Path::new(index_dir)).expect("Could not get indexer");
    let pages_dir = matches.value_of("pages-dir").unwrap_or("./pages");
    let config = Config {
        indiv_crate_dir: Path::new(&pages_dir).join("crates").into(),
        letters_dir: Path::new(&pages_dir).join("letters").into(),
    };
    rocket::ignite()
           .manage(Arc::new(Mutex::new(indexer)))
           .manage(config)
           .attach(Template::fairing())
           .mount("/", routes![index, cr8, search, letter])
           .launch();
}
