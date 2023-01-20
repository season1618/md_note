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
        println!("could not open the file.");
    }
}

use crate::Block::*;
use crate::Span::*;

#[derive(Clone)]
enum Block {
    Header { spans: Vec<Span>, level: u32 },
    Paragraph { spans: Vec<Span> },
    LineBreak,
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
            let block = self.parse_block();
            self.elem_list.push(block);
        }
    }

    fn parse_block(&mut self) -> Block {
        let c = self.doc[self.pos];
        if self.expect("# ") {
            return self.parse_header(1);
        }
        if self.expect("## ") {
            return self.parse_header(2);
        }
        if self.expect("### ") {
            return self.parse_header(3);
        }
        if self.expect("#### ") {
            return self.parse_header(4);
        }
        if self.expect("##### ") {
            return self.parse_header(5);
        }
        if self.expect("###### ") {
            return self.parse_header(6);
        }
        return self.parse_paragraph();
    }

    fn parse_header(&mut self, level: u32) -> Block {
        return Header { spans: self.parse_spans(), level: level };
    }

    fn parse_paragraph(&mut self) -> Block {
        return Paragraph { spans: self.parse_spans() };
    }

    fn parse_spans(&mut self) -> Vec<Span> {
        let mut spans = Vec::new();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' || c == '\r' {
                self.pos += 1;
                break;
            }
            if c == '[' {
                spans.push(self.parse_link());
                continue;
            }
            spans.push(self.parse_text());
        }
        return spans;
    }

    // fn parse_span(&mut self) -> Span {
    //     return self.parse_text();
    // }

    fn parse_link(&mut self) -> Span {
        self.consume("[");

        let title = self.parse_link_text();

        self.consume("]");
        self.consume("(");

        let url = self.parse_link_text();

        self.consume(")");

        Link { title, url }
    }

    fn parse_link_text(&mut self) -> String { // do not include []()
        let mut text = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '[' || c == ']' || c == '(' || c == ')' { // redundant
                break;
            }
            text.push(c);
            self.pos += 1;
        }
        text
    }

    fn parse_text(&mut self) -> Span {
        let mut text = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' || c == '\r' {
                self.pos += 1;
                break;
            }
            
            text.push(c);
            self.pos += 1;
        }
        return Text { text };
    }

    fn expect(&mut self, s: &str) -> bool {
        let cs: Vec<char> = s.chars().collect();
        for i in 0..s.len() {
            if self.doc[self.pos + i] != cs[i] {
                return false;
            }
        }
        self.pos += s.len();
        return true;
    }

    fn consume(&mut self, s: &str) {
        let cs: Vec<char> = s.chars().collect();
        for i in 0..s.len() {
            if self.doc[self.pos + i] != cs[i] {
                panic!("syntax error");
            }
        }
        self.pos += s.len();
    }

    fn gen_html(&self, dest: &mut File) {
        writeln!(dest, "<!DOCTYPE html>");
        writeln!(dest, "<html>");
        writeln!(dest, "<head>");
        writeln!(dest, "  <meta charset=\"utf-8\">");
        writeln!(dest, "  <link rel=\"stylesheet\" href=\"./index.css\">");
        writeln!(dest, "  <title></title>");
        writeln!(dest, "</head>");
        writeln!(dest, "<body>");
        
        writeln!(dest, "  <div id=\"wrapper\">");

        writeln!(dest, "    <nav id=\"sidebar\">");
        writeln!(dest, "    </nav>");

        writeln!(dest, "    <div id=\"content\">");
        
        self.gen_content(dest);

        writeln!(dest, "    <div>");

        writeln!(dest, "</body>");
        writeln!(dest, "</html>");
    }

    fn gen_content(&self, dest: &mut File) {
        for block in &self.elem_list {
            match block {
                Header { spans, level } => { self.gen_header(spans, level, dest); },
                Paragraph { spans } => { self.gen_paragraph(spans, dest); },
                _ => {},
            }
        }
    }

    fn gen_header(&self, spans: &Vec<Span>, level: &u32, dest: &mut File) {
        write!(dest, "      <h{}>", *level);
        self.gen_spans(spans, dest);
        writeln!(dest, "</h{}>", *level);
    }

    fn gen_paragraph(&self, spans: &Vec<Span>, dest: &mut File) {
        write!(dest, "      <p>");
        self.gen_spans(spans, dest);
        writeln!(dest, "</p>");
    }

    fn gen_spans(&self, spans: &Vec<Span>, dest: &mut File) {
        for span in spans {
            match span {
                Link { title, url } => { self.gen_link(title, url, dest); },
                Text { text } => { self.gen_text(text, dest); },
                _ => {},
            }
        }
    }

    fn gen_link(&self, title: &String, url: &String, dest: &mut File) {
        write!(dest, "<a href=\"{}\">{}</a>", *url, *title);
    }

    fn gen_text(&self, text: &String, dest: &mut File) {
        write!(dest, "{}", text);
    }
}