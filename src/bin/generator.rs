extern crate walkdir;
extern crate text_crates_io;

use std::io::Write;
use std::env;
use std::path::Path;

use walkdir::{DirEntry, WalkDir, WalkDirIterator};

use text_crates_io::errors::*;
use text_crates_io::crates_io_client::Crate;
use text_crates_io::index::Indexer;


fn main() {
    let args = env::args().skip(1).take(1).collect::<Vec<_>>();
    let index_dir = match args.get(0) {
        Some(s) => s.to_owned(),
        None => "./crates.io-index".to_string(),
    };

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

    let walker = WalkDir::new(index_dir).into_iter();
    let indexer = Indexer::new(Path::new("./index")).unwrap();

    for entry in walker.filter_entry(|e| !is_config_json(e))
                       .filter_entry(|e| !is_hidden_file(e))
                       .take(10)
    {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => {
                println!("Couldn't read filename {}", path.display());
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
        indexer.add_crate(info);
        print!(".");
        let _ = ::std::io::stdout().flush();
    }
    println!("");
}

