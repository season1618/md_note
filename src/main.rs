use std::io::{BufRead, BufReader, Write};
use std::env;
use std::fs::File;

fn main(){
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    
    let mut conv = Convertor::new();
    conv.convert(file_path);
}

struct Convertor {
    output_file: File,
}

impl Convertor {
    fn new() -> Self {
        Convertor { output_file: File::create("test.html").unwrap() }
    }
    fn convert(&mut self, file_path: &str) {
        match File::open(file_path) {
            Ok(input_file) => {
                for result in BufReader::new(input_file).lines() {
                    if let Ok(s) = result {
                        self.convert_line(s);
                    } else {
                        println!("could not read a line.\n");
                    }
                }
                
            },
            Err(err) => {
                println!("could not open {}: {}", file_path, err);
            },
        }
    }

    fn convert_line(&mut self, s: String) {
        self.output_file.write_all(s.as_bytes()).unwrap();
        self.output_file.write_all(b"\n").unwrap();
    }
}