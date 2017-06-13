use std::path::Path;

use tantivy::Index;
use tantivy::schema::*;

use errors::*;
use crates_io_client::Crate;

pub struct Indexer {
    schema: Schema,
    index: Index,
}

impl Indexer {
    pub fn new(index_path: &Path) -> Result<Indexer> {
        let mut builder = SchemaBuilder::default();
        builder.add_text_field("name", STRING | STORED);
        builder.add_text_field("version", STRING | STORED);
        builder.add_text_field("category", STRING);
        builder.add_text_field("keyword", STRING);
        builder.add_text_field("description", TEXT);
        let schema = builder.build();

        Ok(Indexer {
            schema: schema.clone(),
            index: match Index::create(index_path, schema) {
                Ok(i) => i,
                Err(e) => bail!("{:?}", e),
            },
        })
    }

    pub fn add_crate(&self, cr8: Crate) -> Result<()> {
        let mut index_writer = match self.index.writer(50_000_000) {
            Ok(i) => i,
            Err(e) => bail!("{:?}", e),
        };
        let name = self.schema.get_field("name").unwrap();
        let version = self.schema.get_field("version").unwrap();
        let category_field = self.schema.get_field("category").unwrap();
        let keyword_field = self.schema.get_field("keyword").unwrap();
        let description = self.schema.get_field("description").unwrap();

        let mut doc = Document::default();
        doc.add_text(name, &cr8.crate_.name);
        doc.add_text(version, &cr8.crate_.max_version);
        if let Some(categories) = cr8.categories {
            for category in categories {
                doc.add_text(category_field, &category.category);
            }
        }

        for keyword in cr8.keywords {
            doc.add_text(keyword_field, &keyword.keyword);
        }

        if let Some(desc) = cr8.crate_.description {
            doc.add_text(description, &desc);
        }

        index_writer.add_document(doc);
        index_writer.commit();
        Ok(())
    }
}
