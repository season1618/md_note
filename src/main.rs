use std::io::{BufRead, BufReader, Write};
use std::env;
use std::fs::File;

// macro_rules! writeln {
//     // ( $ dst : expr , $ ( $ arg : tt ) * ) => { ... };
//     ( $arg1: tt, $arg2: tt ) => { write!($arg1, $arg2, "") };
// }

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
                write!(self.output_file, "<!DOCTYPE html>\n");
                write!(self.output_file, "<html>\n");
                write!(self.output_file, "<head>\n");
                write!(self.output_file, "  <meta charset=\"utf-8\">\n");
                write!(self.output_file, "  <link rel=\"stylesheet\" href=\"./index.css\">\n");
                write!(self.output_file, "  <title></title>\n");
                write!(self.output_file, "</head>\n");
                write!(self.output_file, "<body>\n");
                
                write!(self.output_file, "  <div id=\"wrapper\">\n");

                write!(self.output_file, "    <nav id=\"sidebar\">\n");
                write!(self.output_file, "    </nav>\n");

                write!(self.output_file, "    <div id=\"content\">\n");

                for result in BufReader::new(input_file).lines() {
                    if let Ok(s) = result {
                        self.convert_line(s);
                    } else {
                        println!("could not read a line.\n");
                        break;
                    }
                }

                write!(self.output_file, "    <div>\n");

                write!(self.output_file, "</body>\n");
                write!(self.output_file, "</html>\n");
                
            },
            Err(err) => {
                println!("could not open {}: {}", file_path, err);
            },
        }
    }

    fn convert_line(&mut self, s: String) {
        write!(self.output_file, "      <p>{}</p>\n", s);
    }
}