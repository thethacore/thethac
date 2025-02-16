mod parser;

fn main() {
    match parser::ThethaCoreConfig::parse_from_file("example.thtc") {
        Ok(config) => println!("{:#?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
