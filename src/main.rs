pub mod data;
pub mod parser;
pub mod template;
pub mod codegen;

use std::env;
use std::fs;
use std::fs::File;

use crate::parser::parse_markdown;
use crate::template::read_template;
use crate::codegen::gen_html;

fn main(){
    let args: Vec<String> = env::args().collect();
    let src_path = &args[1];
    let dest_path = &args[2];

    let Ok(doc) = fs::read_to_string(src_path) else {
        println!("could not open the source file.");
        return;
    };

    let (title, toc, content) = parse_markdown(&doc);

    let Ok(template) = read_template(dest_path) else {
        println!("could not open or read the destination file.");
        return;
    };

    let Ok(mut dest) = File::create(dest_path) else {
        println!("could not open or create the destination file.");
        return;
    };
    
    let Ok(_) = gen_html(&mut dest, &title, &toc, &content, &template) else {
        println!("could not write to the destination file.");
        return;
    };
}