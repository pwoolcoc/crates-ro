use std::path::Path;

use tantivy::{Index, IndexWriter};
use tantivy::schema::*;
use tantivy::collector::TopCollector;
use tantivy::query::QueryParser;

use errors::*;
use crates_io_client::Crate;

pub struct Indexer {
    schema: Schema,
    index: Index,
    query_parser: QueryParser
}

impl Indexer {
    pub fn new<P: AsRef<Path>>(index_path: P) -> Result<Indexer> {
        let mut builder = SchemaBuilder::default();
        builder.add_text_field("name", TEXT | STORED);
        builder.add_text_field("version", TEXT | STORED);
        builder.add_text_field("category", TEXT) ;
        builder.add_text_field("keyword", TEXT);
        builder.add_text_field("description", TEXT | STORED);
        let schema = builder.build();

        let name = schema.get_field("name").unwrap();
        let version = schema.get_field("version").unwrap();
        let category_field = schema.get_field("category").unwrap();
        let keyword_field = schema.get_field("keyword").unwrap();
        let description = schema.get_field("description").unwrap();

        let query_parser = QueryParser::new(schema.clone(), vec![name, version,
                                                                 category_field, keyword_field,
                                                                 description]);

        Ok(Indexer {
            schema: schema.clone(),
            index: Index::create(index_path.as_ref(), schema)?,
            query_parser: query_parser,
        })
    }

    pub fn searcher<P: AsRef<Path>>(index_path: P) -> Result<Indexer> {
        let index = Index::open(index_path.as_ref())?;
        let schema = index.schema().clone();

        let name = schema.get_field("name").unwrap();
        let version = schema.get_field("version").unwrap();
        let category_field = schema.get_field("category").unwrap();
        let keyword_field = schema.get_field("keyword").unwrap();
        let description = schema.get_field("description").unwrap();

        let query_parser = QueryParser::new(schema.clone(), vec![name, version,
                                                                 category_field, keyword_field,
                                                                 description]);
        Ok(Indexer {
            schema: schema.clone(),
            index: index,
            query_parser: query_parser,
        })
    }

    pub fn writer(&self) -> Result<IndexWriter> {
        Ok(self.index.writer(50_000_000)?)
    }

    pub fn search(&self, query: &str) -> Result<Vec<(String, String, Option<String>)>> {
        self.index.load_searchers()?;
        let schema = self.index.schema().clone();

        let searcher = self.index.searcher();
        let query = match self.query_parser.parse_query(query) {
            Ok(qp) => qp,
            Err(e) => bail!("Could not parse query: {:?}", e),
        };

        let mut top_collector = TopCollector::with_limit(10);

        searcher.search(&*query, &mut top_collector)?;

        let doc_addrs = top_collector.docs();
        let results = doc_addrs.iter().filter_map(|addr| searcher.doc(addr).ok());

        let results = results.map(|doc| {
            let name = doc.get_first(
                    schema.get_field("name").unwrap()).unwrap().text().to_owned();
            let version = doc.get_first(
                    schema.get_field("version").unwrap()).unwrap().text().to_owned();
            let description = doc.get_first(
                    schema.get_field("description").unwrap()).map(|f| f.text().to_owned());
            (name, version, description)
        }).collect();

        Ok(results)
    }

    pub fn add_crate(&self, cr8: &Crate, index_writer: &mut IndexWriter) -> Result<()> {

        let name = self.schema.get_field("name").unwrap();
        let version = self.schema.get_field("version").unwrap();
        let category_field = self.schema.get_field("category").unwrap();
        let keyword_field = self.schema.get_field("keyword").unwrap();
        let description = self.schema.get_field("description").unwrap();

        let mut doc = Document::default();
        doc.add_text(name, &cr8.crate_.name);
        doc.add_text(version, &cr8.crate_.max_version);
        if let Some(ref categories) = cr8.categories {
            for category in categories {
                doc.add_text(category_field, &category.category);
            }
        }

        for keyword in &cr8.keywords {
            doc.add_text(keyword_field, &keyword.keyword);
        }

        if let Some(ref desc) = cr8.crate_.description {
            doc.add_text(description, &desc);
        }

        index_writer.add_document(doc);
        Ok(())
    }
}
