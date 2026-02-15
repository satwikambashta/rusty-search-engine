use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;
use tiny_http::{Header, Response, Server};
use crate::model::InvertedIndex;
use crate::lexer::Lexer;
use crate::ranking::{compute_idf, compute_tf};

pub fn serve_files(index: Arc<InvertedIndex>) -> io::Result<()> {
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
