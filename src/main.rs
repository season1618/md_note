use std::io::Write;
use std::env;
use std::fs::read_to_string;
use std::fs::File;

fn main(){
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let mut dest = File::create("test.html").unwrap();

    if let Ok(content) = read_to_string(file_path) {
        let mut conv = Convertor::new(content);
        conv.parse_markdown();
        conv.gen_html(&mut dest);
    } else {
        println!("could not open the file.\n");
    }

    // match File::open(file_path) {
    //     Ok(src) => {
    //         for result in BufReader::new(src).lines() {
    //             if let Ok(s) = result {
    //                 conv.parse_block(s);
    //             } else {
    //                 println!("could not read a line.\n");
    //                 break;
    //             }
    //         }
    //     },
    //     Err(err) => {
    //         println!("could not open {}: {}", file_path, err);
    //     },
    // }
}

use crate::Block::*;
use crate::Span::*;

#[derive(Clone)]
enum Block {
    Paragraph { spans: Vec<Span> },
    LineBreak,
    Header { spans: Vec<Span>, level: i32 },
    Blockquote { spans: Vec<Span> },
    List { items: Vec<ListItem> },
    CodeBlock { code: String },
}

#[derive(Clone)]
enum Span {
    Text { text: String },
    Link { title: String, url: String },
    Emphasis { kind: EmphasisKind, spans: Vec<Span> },
    Code { code: String },
    Image { url: String },
}

#[derive(Clone)]
struct ListItem {
    spans: Vec<Span>,
    list: Option<Span>,
}

#[derive(Clone)]
enum EmphasisKind {
    Em,
    Strong,
}

struct Convertor {
    doc: Vec<char>,
    pos: usize,
    elem_list: Vec<Block>,
}

impl Convertor {
    fn new(doc: String) -> Self {
        Convertor {
            doc: doc.chars().collect(),
            pos: 0,
            elem_list: Vec::new(),
        }
    }

    fn parse_markdown(&mut self) {
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' {
                self.pos += 1;
                continue;
            }

            self.parse_block();
        }
    }

    fn parse_block(&mut self) {
        self.parse_paragraph();
    }

    fn parse_paragraph(&mut self) {
        let mut text = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' {
                self.elem_list.push(Paragraph { spans: vec![ Text { text } ] });
                self.pos += 1;
                return;
            }
            
            text.push(c);
            self.pos += 1;
        }
    }

    fn gen_html(&self, dest: &mut File) {
        write!(dest, "<!DOCTYPE html>\n");
        write!(dest, "<html>\n");
        write!(dest, "<head>\n");
        write!(dest, "  <meta charset=\"utf-8\">\n");
        write!(dest, "  <link rel=\"stylesheet\" href=\"./index.css\">\n");
        write!(dest, "  <title></title>\n");
        write!(dest, "</head>\n");
        write!(dest, "<body>\n");
        
        write!(dest, "  <div id=\"wrapper\">\n");

        write!(dest, "    <nav id=\"sidebar\">\n");
        write!(dest, "    </nav>\n");

        write!(dest, "    <div id=\"content\">\n");
        
        self.gen_content(dest);

        write!(dest, "    <div>\n");

        write!(dest, "</body>\n");
        write!(dest, "</html>\n");
    }

    fn gen_content(&self, dest: &mut File) {
        for block in &self.elem_list {
            match block {
                Paragraph { spans } => { self.gen_paragraph(spans, dest); },
                _ => {},
            }
        }
    }

    fn gen_paragraph(&self, spans: &Vec<Span>, dest: &mut File) {
        for span in spans {
            match span {
                Text { text } => { write!(dest, "      <p>{}</p>\n", *text); },
                _ => {},
            }
        }
    }
}