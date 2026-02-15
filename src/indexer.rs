use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use xml::reader::{EventReader, XmlEvent};
use crate::model::DocStats;
use crate::lexer::Lexer;

pub fn index_document(content: &str) -> DocStats {
    let chars: Vec<char> = content.chars().collect();
    let lexer = Lexer::new(&chars);
    let mut tf = HashMap::new();
    let mut total_words = 0;

    for token in lexer {
        if crate::stopwords::is_stopword(&token) {
            continue;
        }
        *tf.entry(token).or_insert(0) += 1;
        total_words += 1;
    }

    DocStats { tf, total_words }
}

pub fn parse_xml_file(file_path: &Path) -> io::Result<String> {
    let file = fs::File::open(file_path)?;
    let reader = EventReader::new(io::BufReader::new(file));

    let mut buffer = String::new();
    for event in reader {
        match event {
            Ok(XmlEvent::Characters(_doc_content)) => {
                buffer.push_str(&_doc_content);
                buffer.push(' ');
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("Failed to read event in {}: {err}", file_path.display());
                return Err(io::Error::new(io::ErrorKind::Other, err));
            }
        }
    }

    Ok(buffer)
}

pub fn traverse_directory(dir_path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(traverse_directory(&path)?);
            } else if path.extension().map_or(false, |ext| ext == "xhtml") {
                files.push(path);
            }
        }
    }
    Ok(files)
}
