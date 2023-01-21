pub mod convertor;

use std::env;
use std::fs;
use std::fs::File;

use crate::convertor::Convertor;

fn main(){
    let args: Vec<String> = env::args().collect();
    let src_path = &args[1];
    let dest_path = &args[2];

    let mut dest = File::create(dest_path).unwrap();

    if let Ok(content) = fs::read_to_string(src_path) {
        let mut conv = Convertor::new(content);
        conv.parse_markdown();
        conv.gen_html(&mut dest);
    } else {
        println!("could not open the file.");
    }
}