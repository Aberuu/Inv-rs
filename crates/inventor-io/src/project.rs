use inventor_core::Document;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

pub struct ProjectFile;

impl ProjectFile {
    pub fn save<P: AsRef<Path>>(doc: &Document, path: P) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, doc)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Document> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let doc: Document = serde_json::from_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(doc)
    }
}
