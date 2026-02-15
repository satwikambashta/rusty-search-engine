use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;
use tiny_http::{Header, Response, Server};
use xml::reader::{EventReader, XmlEvent};
use rayon::prelude::*;

#[derive(Serialize, Deserialize, Debug, Default)]
struct DocStats {
    tf: HashMap<String, usize>,
    total_words: usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct InvertedIndex {
    docs: HashMap<PathBuf, DocStats>,
}

struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        while !self.content.is_empty() && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn next_token(&mut self) -> Option<String> {
        self.trim_left();
        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_alphanumeric() {
            let mut n = 0;
            while n < self.content.len() && self.content[n].is_alphanumeric() {
                n += 1;
            }
            let token = self.content[..n].iter().collect::<String>().to_uppercase();
            self.content = &self.content[n..];
            return Some(token);
        }

        // non-alphanumeric single chars (punctuation)
        self.content = &self.content[1..];
        self.next_token()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn index_document(content: &str) -> DocStats {
    let chars: Vec<char> = content.chars().collect();
    let lexer = Lexer::new(&chars);
    let mut tf = HashMap::new();
    let mut total_words = 0;

    for token in lexer {
        *tf.entry(token).or_insert(0) += 1;
        total_words += 1;
    }

    DocStats { tf, total_words }
}

fn parse_xml_file(file_path: &Path) -> io::Result<String> {
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

fn traverse_directory(dir_path: &Path) -> io::Result<Vec<PathBuf>> {
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

fn compute_tf(term: &str, doc: &DocStats) -> f32 {
    let n = doc.total_words as f32;
    if n == 0.0 { return 0.0; }
    let f = *doc.tf.get(term).unwrap_or(&0) as f32;
    f / n
}

fn compute_idf(term: &str, index: &InvertedIndex) -> f32 {
    let n = index.docs.len() as f32;
    let m = index.docs.values().filter(|stats| stats.tf.contains_key(term)).count() as f32;
    (n / (1.0 + m)).log10()
}

fn serve_files(index: Arc<InvertedIndex>) -> io::Result<()> {
    let server = Server::http("0.0.0.0:6969").unwrap();
    println!("Server running on http://localhost:6969");

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        
        if url == "/" {
            let content = fs::read_to_string("index.html").unwrap_or_else(|_| "<h1>Index.html not found</h1>".to_string());
            let response = Response::from_string(content).with_header(
                Header::from_bytes("Content-Type", "text/html").unwrap()
            );
            request.respond(response)?;
            continue;
        }

        if url.starts_with("/api/search") {
            let query_string = url.split('?').nth(1).unwrap_or("");
            let mut search_terms = Vec::new();

            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    if key == "q" {
                        if let Some(value) = parts.next() {
                            let decoded = value.replace("+", " ").replace("%20", " ");
                            let chars: Vec<char> = decoded.chars().collect();
                            let lexer = Lexer::new(&chars);
                            for token in lexer {
                                search_terms.push(token);
                            }
                        }
                    }
                }
            }
            
            println!("Searching for: {:?}", search_terms);

            let mut results: Vec<(String, f32, String)> = Vec::new();
            
            // precompute
            let idfs: HashMap<String, f32> = search_terms.iter().map(|t| {
                (t.clone(), compute_idf(t, &index))
            }).collect();

            for (path, stats) in &index.docs {
                let mut score = 0.0;
                let mut first_match = None;
                for term in &search_terms {
                    let tf = compute_tf(term, stats);
                    let idf = *idfs.get(term).unwrap_or(&0.0);
                    score += tf * idf;
                    
                    if first_match.is_none() && stats.tf.contains_key(term) {
                        first_match = Some(term.clone());
                    }
                }
                
                if score > 0.0 {
                    let mut snippet = String::new();
                    if let Some(term) = first_match {
                        if let Ok(content) = fs::read_to_string(path) {
                            let content_upper = content.to_uppercase();
                            if let Some(pos) = content_upper.find(&term) {
                                // char boundaries
                                let char_pos = content[..pos].chars().count();
                                
                                let start_char = char_pos.saturating_sub(15);
                                let end_char = (char_pos + term.chars().count() + 40).min(content.chars().count());
                                
                                snippet = content.chars().skip(start_char).take(end_char - start_char).collect::<String>();
                                snippet = snippet.replace('\n', " ").trim().to_string();
                                
                                if start_char > 0 { snippet = format!("...{}", snippet); }
                                if end_char < content.chars().count() { snippet = format!("{}...", snippet); }
                            }
                        }
                    }
                    results.push((path.display().to_string().replace('\\', "/"), score, snippet));
                }
            }

            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            results.truncate(20);

            let json_results = serde_json::to_string(&results)?;
            let response = Response::from_string(json_results)
                .with_header(Header::from_bytes("Content-Type", "application/json").unwrap())
                .with_header(Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap());
            
            request.respond(response)?;
            continue;
        }

        if url.starts_with("/docs/") {
            let file_path_str = &url["/docs/".len()..];
            let decoded_path = urlencoding::decode(file_path_str)
                .unwrap_or_else(|_| file_path_str.to_string().into());
            let file_path = Path::new(decoded_path.as_ref());

            if file_path.exists() && file_path.is_file() {
                match fs::read_to_string(file_path) {
                    Ok(content) => {
                        let content_type = if file_path.extension().map_or(false, |e| e == "xhtml") {
                            "application/xhtml+xml"
                        } else {
                            "text/html"
                        };
                        let response = Response::from_string(content)
                            .with_header(Header::from_bytes("Content-Type", content_type).unwrap());
                        request.respond(response)?;
                    }
                    Err(_) => {
                        let response = Response::from_string("Failed to read file").with_status_code(500);
                        request.respond(response)?;
                    }
                }
            } else {
                let response = Response::from_string("File not found").with_status_code(404);
                request.respond(response)?;
            }
            continue;
        }

        let response = Response::from_string("Not Found").with_status_code(404);
        request.respond(response)?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: \n  cargo run index <folder_path>\n  cargo run serve");
        exit(1);
    }

    let mode = &args[1];

    if mode == "serve" {
        let index: InvertedIndex = if let Ok(file) = fs::File::open("index.json") {
            serde_json::from_reader(io::BufReader::new(file)).unwrap_or_default()
        } else {
            eprintln!("index.json not found. Please run indexing first.");
            InvertedIndex::default()
        };
        serve_files(Arc::new(index))?;
    } else if mode == "index" {
        if args.len() < 3 {
             eprintln!("Usage: cargo run index <folder_path>");
             exit(1);
        }
        let directory_path = Path::new(&args[2]);
        let files = traverse_directory(directory_path)?;
        
        println!("Indexing {} files...", files.len());
        
        let docs: HashMap<PathBuf, DocStats> = files
            .par_iter()
            .filter_map(|path| {
                match parse_xml_file(path) {
                    Ok(content) => {
                        println!("Indexing: {}", path.display());
                        Some((path.clone(), index_document(&content)))
                    }
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        let index = InvertedIndex { docs };

        println!("\n--- Summary ---");
        println!("Total documents indexed: {}", index.docs.len());

        let file = fs::File::create("index.json")?;
        serde_json::to_writer_pretty(file, &index)?;
        println!("Index saved to index.json");
    } else {
        eprintln!("Unknown mode: {}", mode);
        eprintln!("Usage: \n  cargo run index <folder_path>\n  cargo run serve");
        exit(1);
    }

    Ok(())
}
