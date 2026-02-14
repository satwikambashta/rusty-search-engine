use std::env;
use std::fs;
use std::io;
use std::process::exit;
use std::path::Path;
use xml::reader::{XmlEvent, EventReader};
use std::collections::HashMap;

struct Lexer<'a> {
    _doc_content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(_doc_content: &'a [char]) -> Self {
        Self { _doc_content }
    }

    fn trim_left(&mut self) {
        while self._doc_content.len() > 0 && self._doc_content[0].is_whitespace() {
            self._doc_content = &self._doc_content[1..];
        }
    }

    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();
        if self._doc_content.len() == 0 {
            return None;
        }

       if self._doc_content[0].is_alphanumeric() {
            let mut n = 0;
            while n < self._doc_content.len() && self._doc_content[n].is_alphanumeric() {
                n += 1;
            }
            let token = &self._doc_content[..n];
            self._doc_content = &self._doc_content[n..];
            return Some(token);
        }

        let token = &self._doc_content[..1];
        self._doc_content = &self._doc_content[1..];
        Some(token)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn index_document(_doc_content: &[char]) -> HashMap<String, usize> {
    let mut index = HashMap::new();
    let lexer = Lexer::new(_doc_content);
    for token in lexer {
        let word: String = token.iter().collect();
        let word = word.to_lowercase();
        //punctuation
        if word.chars().all(|c| c.is_alphanumeric()) {
            *index.entry(word).or_insert(0) += 1;
        }
    }
    index
}

fn parse_xml_file(file_path: &Path) -> io::Result<String> {
    let file = fs::File::open(file_path)?;
    let reader = EventReader::new(io::BufReader::new(file));

    let mut buffer = String::new();
    for event in reader {
        let event = event.unwrap_or_else(|err| {
            eprintln!("Failed to read event: {err}");
            exit(1);
        });
        if let XmlEvent::Characters(_doc_content) = event {
            buffer.push_str(&_doc_content);
            buffer.push(' ');
        }
    }

    Ok(buffer)
}

fn main() -> io::Result<()> {
    let directory_path = "docs.gl/gl4";
    let mut all_documents: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for entry in fs::read_dir(directory_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            match parse_xml_file(&path) {
                Ok(buffer) => {
                    let _doc_content: Vec<char> = buffer.chars().collect();
                    let index = index_document(&_doc_content);
                    println!(
                        "Indexed: {} ({} unique tokens, {} total chars)",
                        path.display(),
                        index.len(),
                        _doc_content.len()
                    );
                    all_documents.insert(path.display().to_string(), index);
                }
                Err(e) => {
                    eprintln!("Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    println!("\n--- Summary ---");
    println!("Total documents indexed: {}", all_documents.len());

    Ok(())
}
