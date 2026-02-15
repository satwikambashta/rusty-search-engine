use std::env;
use std::fs;
use std::io;
use std::process::exit;
use std::path::Path;
use xml::reader::{XmlEvent, EventReader};
use std::collections::HashMap;
use tiny_http::{Server, Response, Header};
use std::sync::Arc;

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
        let word = word.to_uppercase();
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

fn traverse_directory(
    dir_path: &Path,
    all_documents: &mut HashMap<String, HashMap<String, usize>>
) -> io::Result<()> {
    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                traverse_directory(&path, all_documents)?;
            } else {
                let path_str = path.display().to_string();
                if all_documents.contains_key(&path_str) {
                    // println!("Skipping already indexed file: {}", path_str);
                    continue;
                }

                if let Some(extension) = path.extension() {
                    if extension == "xhtml" {
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
                                let mut top_tokens: Vec<(String, usize)> = index.into_iter().collect();
                                top_tokens.sort_by(|a, b| b.1.cmp(&a.1));
                                top_tokens.truncate(10);
                                
                                let top_index: HashMap<String, usize> = top_tokens.into_iter().collect();
                                all_documents.insert(path_str, top_index);
                            }
                            Err(e) => {
                                eprintln!("Failed to parse {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn serve_files(index: HashMap<String, HashMap<String, usize>>) -> io::Result<()> {
    let server = Server::http("0.0.0.0:6969").unwrap();
    println!("Server running on http://localhost:6969");
    let index = Arc::new(index);

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        println!("Request: {}", url);

        if url == "/" {
            let content = fs::read_to_string("index.html").unwrap_or_else(|_| "<h1>Index.html not found</h1>".to_string());
            let response = Response::from_string(content).with_header(
                Header::from_bytes("Content-Type", "text/html").unwrap()
            );
            request.respond(response)?;
            continue;
        }

        if url.starts_with("/api/search") {
            // query param parsing
            let query_string = url.split('?').nth(1).unwrap_or("");
            let mut search_terms = Vec::new();

            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    if key == "q" {
                        if let Some(value) = parts.next() {
                            // Decode URL encoding roughly (replace + with space, %20 with space)
                            let decoded = value.replace("+", " ").replace("%20", " ");
                            let chars: Vec<char> = decoded.chars().collect();
                            let lexer = Lexer::new(&chars);
                            for token in lexer {
                                search_terms.push(token.iter().collect::<String>().to_uppercase());
                            }
                        }
                    }
                }
            }
            
            println!("Searching for: {:?}", search_terms);

            // search logic
            let mut results: Vec<(String, usize)> = Vec::new();
            
            for (doc_path, doc_index) in index.iter() {
                let mut score = 0;
                for term in &search_terms {
                    for (key, count) in doc_index {
                        if key.contains(term) {
                            score += count;
                        }
                    }
                }
                if score > 0 {
                    results.push((doc_path.clone(), score));
                }
            }

            results.sort_by(|a, b| b.1.cmp(&a.1));
            // Limit to top 20
            if results.len() > 20 {
                results.truncate(20);
            }

            let json_results = serde_json::to_string(&results)?;
            let response = Response::from_string(json_results)
                .with_header(Header::from_bytes("Content-Type", "application/json").unwrap())
                .with_header(Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap());
            
            request.respond(response)?;
            continue;
        }

        // 404
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
        let index: HashMap<String, HashMap<String, usize>> =
            if let Ok(file) = fs::File::open("index.json") {
                serde_json::from_reader(io::BufReader::new(file)).unwrap_or_else(|_| HashMap::new())
            } else {
                eprintln!("index.json not found. Please run indexing first.");
                HashMap::new()
            };
        serve_files(index)?;
    } else if mode == "index" {
        if args.len() < 3 {
             eprintln!("Usage: cargo run index <folder_path>");
             exit(1);
        }
        let directory_path = Path::new(&args[2]);

        let mut all_documents: HashMap<String, HashMap<String, usize>> =
            if let Ok(file) = fs::File::open("index.json") {
                serde_json::from_reader(io::BufReader::new(file)).unwrap_or_else(|_| HashMap::new())
            } else {
                HashMap::new()
            };

        traverse_directory(directory_path, &mut all_documents)?;

        println!("\n--- Summary ---");
        println!("Total documents indexed: {}", all_documents.len());

        let file = fs::File::create("index.json")?;
        serde_json::to_writer_pretty(file, &all_documents)?;
        println!("Index saved to index.json");
    } else {
        eprintln!("Unknown mode: {}", mode);
        eprintln!("Usage: \n  cargo run index <folder_path>\n  cargo run serve");
        exit(1);
    }

    Ok(())
}
