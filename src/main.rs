mod model;
mod lexer;
mod indexer;
mod ranking;
mod server;
mod stopwords;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;
use rayon::prelude::*;

use crate::model::InvertedIndex;
use crate::indexer::{index_document, parse_xml_file, traverse_directory};
use crate::server::serve_files;

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
        
        let docs: HashMap<PathBuf, model::DocStats> = files
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
