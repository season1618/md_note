pub mod data;
pub mod parser;
pub mod codegen;

use std::env;
use std::fs;
use std::fs::File;

use crate::parser::parse_markdown;
use crate::codegen::gen_html;

fn main(){
    let args: Vec<String> = env::args().collect();
    let src_path = &args[1];
    let dest_path = &args[2];

    let mut dest = File::create(dest_path).unwrap();

    if let Ok(doc) = fs::read_to_string(src_path) {
        let (title, toc, content) = parse_markdown(doc);
        gen_html(&mut dest, title, toc, content);
    } else {
        println!("could not open the file.");
    }
}