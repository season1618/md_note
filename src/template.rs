use std::io::{self, BufRead, BufReader};
use std::fs::File;
use regex::Regex;

use crate::data::Elem;
use Elem::*;

pub fn read_template(dest_path: &str) -> Result<Vec<Elem>, io::Error> {
    let dest = File::open(dest_path)?;
    let mut reader = BufReader::new(dest);
    let mut line = String::new();
    let mut template: Vec<Elem> = Vec::new();

    let regex_title = Regex::new("<title>.*</title>").unwrap();
    let regex_toc_begin = Regex::new("<nav id=\"toc\">").unwrap();
    let regex_toc_end = Regex::new("</nav>").unwrap();
    let regex_content_begin = Regex::new("<div id=\"content\">").unwrap();
    let regex_content_end = Regex::new("</div>").unwrap();

    while reader.read_line(&mut line)? > 0 {
        if let Some(m) = regex_title.find(&line) {
            template.push(Title(m.start()));
        } else if let Some(m) = regex_toc_begin.find(&line) {
            template.push(Toc(m.start()));
            while reader.read_line(&mut line)? > 0 {
                if regex_toc_end.is_match(&line) { break; }
                line.clear();
            }
        } else if let Some(m) = regex_content_begin.find(&line) {
            template.push(Content(m.start()));
            while reader.read_line(&mut line)? > 0 {
                if regex_content_end.is_match(&line) { break; }
                line.clear();
            }
        } else {
            template.push(Str(line.clone()));
        }
        line.clear();
    }

    Ok(template)
}