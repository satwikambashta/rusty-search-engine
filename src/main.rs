use std::env;
use std::fs;
use std::io;
use std::process::exit;
use xml::reader::{XmlEvent,EventReader};

fn parse_xml_file(file_path: &str) -> io::Result<String> {
    let file = fs::File::open(file_path).unwrap_or_else(|err| {
        eprintln!("Failed to read file: {err}");
        exit(1);
    });
    let reader = EventReader::new(file);

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

fn main() {
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     eprintln!("Usage: {} <file_path>", args[0]);
    //     exit(1);
    // }
    // let file_path = &args[1];
    let file_path = "docs.gl/gl4/glClear.xhtml";
    println!("Content: {buffer}", buffer=parse_xml_file(file_path).unwrap());
    
}
