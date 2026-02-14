use std::env;
use std::fs;
use std::process::exit;

fn main() {
    let file_path = "docs.gl/gl4/glClear.xhtml";
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     eprintln!("Usage: {} <file_path>", args[0]);
    //     exit(1);
    // }
    // let file_path = &args[1];
    let contents = fs::read_to_string(file_path).unwrap_or_else(|err| {
        eprintln!("Failed to read file: {err}");
        String::new()
    });
    println!("Length of file: {}", contents.len());
    println!("With text:\n{contents}");
}
