use std::env;
use std::fs;
use std::io;
use std::process::exit;
use std::path::Path;
use xml::reader::{XmlEvent, EventReader};
use std::collections::HashMap;

struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self {
            content
        }
    }
}

fn index_document(_doc_content: &str) -> HashMap<String, usize> {
    let mut index = HashMap::new();
    for word in _doc_content.split_whitespace() {
        *index.entry(word.to_string()).or_insert(0) += 1;
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
        if let XmlEvent::Characters(content) = event {
            buffer.push_str(&content);
        }
    }

    let contents = fs::read_to_string(file_path).unwrap_or_else(|err| {
        eprintln!("Failed to read file: {err}");
        String::new()
    });
    Ok(buffer)
}

fn main() -> io::Result<()> {
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     eprintln!("Usage: {} <file_path>", args[0]);
    //     exit(1);
    // }
    // let file_path = &args[1];
    let content = parse_xml_file(Path::new("docs.gl/gl4/glMemoryBarrier.xhtml"))?.chars().collect::<Vec<_>>();
    let lexer = Lexer::new(&content);
    println!("{:?}", lexer.content);
    // let mut all_documents = HashMap<Path, HashMap<String, usize>>::new();
    // let directory_path = "docs.gl/gl4";
    // for entry in fs::read_dir(directory_path)? {
    //     let entry = entry?;
    //     let path = entry.path();

    //     if path.is_file() {
    //         match parse_xml_file(&path) {
    //             Ok(buffer) => {
    //                 println!("Content size: {} File: {}", buffer.len(), path.display());
    //             },
    //             Err(e) => {
    //                 eprintln!("Failed to parse {}: {}", path.display(), e);
    //             }
    //         }
    //     }
    // }
    Ok(())
}
