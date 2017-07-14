use std::path::{PathBuf, Path};
use std::io::Write;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};

use errors::*;
use crates_io_client::Crate;

pub struct PageGenerator {
    crates_dir: PathBuf,
    letters_dir: PathBuf
}

pub fn directory_for_name(name: &str) -> PathBuf {
    let len = name.len();
    if len == 1 {
        Path::new("1").to_path_buf()
    } else if len == 2 {
        Path::new("2").to_path_buf()
    } else if len == 3 {
        Path::new("3").to_path_buf()
    } else {
        Path::new(&name[..2]).join(
                Path::new(&name[2..4]))
    }
}

impl PageGenerator {
    pub fn new<P: Into<PathBuf>>(page_dir: P) -> PageGenerator {
        let page_dir = page_dir.into();
        PageGenerator {
            crates_dir: page_dir.join("crates"),
            letters_dir: page_dir.join("letters"),
        }
    }

    pub fn generate_letters(&self, letters: HashMap<String, Vec<(String, String)>>) -> Result<()> {
        #[derive(Serialize, Deserialize)]
        struct Names {
            pub names: Vec<Pair>,
        }

        #[derive(Serialize, Deserialize)]
        struct Pair {
            pub name: String,
            pub version: String
        }

        let tera = compile_templates!("src/generator-templates/**/*");
        fs::create_dir_all(&self.letters_dir)?;
        for (letter, pairs) in letters.into_iter() {
            let pairs = pairs.into_iter().map(|p| Pair { name: p.0, version: p.1 }).collect::<Vec<_>>();
            let data = Names {
                names: pairs,
            };
            let rendered = match tera.render("letter.html.tera", &data) {
                Ok(r) => r,
                Err(e) => bail!("error: {:?}", e),
            };
            let fname = self.letters_dir.join(letter);
            let mut file = match OpenOptions::new()
                                             .create(true)
                                             .read(true)
                                             .write(true)
                                             .open(&fname) {
                Ok(f) => f,
                Err(e) => bail!("{:?}", e),
            };
            write!(&mut file, "{}", rendered).chain_err(|| "Could not write to file")?;
        }
        Ok(())
    }

    pub fn generate_crate(&self, cr8: &Crate) -> Result<()> {
        let tera = compile_templates!("src/generator-templates/**/*");
        let rendered = match tera.render("crate.html.tera", &cr8.crate_) {
            Ok(r) => r,
            Err(e) => bail!("error: {:?}", e),
        };
        let name = cr8.crate_.name.to_lowercase();
        let path = self.crates_dir.join(directory_for_name(&name));
        fs::create_dir_all(&path)?;
        let fname = path.join(Path::new(&name));
        let mut file = match OpenOptions::new()
                                         .create(true)
                                         .read(true)
                                         .write(true)
                                         .open(&fname) {
            Ok(f) => f,
            Err(e) => bail!("{:?}", e),
        };
        write!(&mut file, "{}", rendered).chain_err(|| "Could not write to file")?;
        Ok(())
    }
}

