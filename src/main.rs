use std::env;
use std::fs;
use std::process::exit;
use xml::reader::{XmlEvent,EventReader};

fn parse_xml_file(file_path: &str) -> io::Result<String> {
    
}

fn main() {
    let file_path = "docs.gl/gl4/glClear.xhtml";
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     eprintln!("Usage: {} <file_path>", args[0]);
    //     exit(1);
    // }
    // let file_path = &args[1];
    let file = fs::File::open(file_path).unwrap_or_else(|err| {
        eprintln!("Failed to read file: {err}");
        exit(1);
    });
    let reader = EventReader::new(file);

    let mut buffer = String::new();
    for event in reader {
        // match event {
        //     Ok(event) => println!("Event: {event:?}"),
        //     Err(err) => eprintln!("Error: {err}"),
        // }
        let event = event.unwrap_or_else(|err| {
            eprintln!("Failed to read event: {err}");
            exit(1);
        });
        if let XmlEvent::Characters(content) = event {
            // println!("Content: {content}");
            buffer.push_str(&content);
        }
        // println!("Event: {event:?}");
    }

    let contents = fs::read_to_string(file_path).unwrap_or_else(|err| {
        // eprintln!("Failed to read file: {err}");
        String::new()
    });
    // println!("Length of file: {}", contents.len());
    // println!("With text:\n{contents}");
    println!("Content: {buffer}");
}
