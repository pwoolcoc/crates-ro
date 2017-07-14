extern crate walkdir;
extern crate crates_no;
extern crate clap;
extern crate tera;

use std::fs;
use std::io::Write;
use std::collections::HashMap;
use std::path::Path;

use clap::{App, Arg};
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

use crates_no::errors::*;
use crates_no::crates_io_client::Crate;
use crates_no::index::Indexer;
use crates_no::page_generator::PageGenerator;

fn main() {
    let matches = App::new("index-generator")
                      .arg(Arg::with_name("INDEX")
                               .index(1)
                               .required(true))
                      .arg(Arg::with_name("crates-index")
                               .takes_value(true))
                      .arg(Arg::with_name("page-dir")
                               .takes_value(true))
                      .get_matches();
    let index_path = Path::new(matches.value_of("INDEX").unwrap());

    if ! index_path.exists() {
        let _ = fs::create_dir_all(index_path);
    }

    let crates_dir = match matches.value_of("crates-index") {
        Some(s) => Path::new(s),
        None => Path::new("./crates.io-index"),
    };

    let page_dir = match matches.value_of("page-dir") {
        Some(s) => Path::new(s),
        None => Path::new("./pages"),
    };
    if let Err(e) = run(index_path, crates_dir, page_dir) {
        eprintln!("error: {}", e);
        ::std::process::exit(1);
    }
}

fn run<P, Q, R>(index_path: P, crates_path: Q, page_dir: R) -> Result<()>
        where P: AsRef<Path>,
              Q: AsRef<Path>,
              R: AsRef<Path>
{
    let index_path = index_path.as_ref();
    let crates_path = crates_path.as_ref();
    let page_dir = page_dir.as_ref();

    fn is_hidden_file(entry: &DirEntry) -> bool {
        entry.file_name()
             .to_str()
             .map(|f| f.starts_with('.'))
             .unwrap_or(false)
    }

    fn is_config_json(entry: &DirEntry) -> bool {
        entry.file_name()
             .to_str()
             .map(|f| f == "config.json")
             .unwrap_or(false)
    }

    let walker = WalkDir::new(crates_path).into_iter();
    let indexer = Indexer::new(index_path).unwrap();
    let page = PageGenerator::new(page_dir);
    let mut letters = HashMap::new();
    let mut writer = indexer.writer()?;

    for entry in walker.filter_entry(|e| !is_config_json(e))
                       .filter_entry(|e| !is_hidden_file(e))
                       .take(2000)
    {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => {
                eprintln!("Couldn't read filename {}", path.display());
                continue;
            },
        };
        let info = match Crate::get(&name) {
            Ok(info) => info,
            Err(Error(ErrorKind::Msg(_), _)) => {
                continue;
            },
            Err(e) => {
                eprintln!("Error getting '{}' info for crate {:?}", name, e);
                continue;
            },
        };
        indexer.add_crate(&info, &mut writer)?;
        let first_letter = info.crate_.name[..1].to_lowercase();
        let name = info.crate_.name.to_string();
        let version = info.crate_.max_version.to_string();
        let letter_vec = letters.entry(first_letter).or_insert(Vec::new());
        letter_vec.push((name, version));
        page.generate_crate(&info)?;
        print!(".");
        let _ = ::std::io::stdout().flush();
    }
    page.generate_letters(letters)?;
    writer.commit()?;
    println!("");
    Ok(())
}

