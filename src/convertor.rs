use std::io::Write;
use std::fs::File;

use Block::*;
use Span::*;
use EmphasisKind::*;

#[derive(Debug)]
enum Block {
    Header { spans: Vec<Span>, level: u32 },
    Blockquote { spans: Vec<Span> },
    List { items: Vec<ListItem> },
    CodeBlock { code: String },
    Paragraph { spans: Vec<Span> },
    LineBreak,
}

#[derive(Clone, Debug)]
enum Span {
    Link { title: String, url: String },
    Emphasis { kind: EmphasisKind, text: String },
    Code { code: String },
    Image { url: String },
    Text { text: String },
}

#[derive(Debug)]
struct ListItem {
    spans: Vec<Span>,
    list: Block,
}

#[derive(Clone, Debug)]
enum EmphasisKind {
    Em,
    Strong,
}

pub struct Convertor {
    doc: Vec<char>,
    pos: usize,
    title: String,
    sidebar: Block,
    content: Vec<Block>,
}

impl Convertor {
    pub fn new(doc: String) -> Self {
        Convertor {
            doc: doc.chars().collect(),
            pos: 0,
            title: "".to_string(),
            sidebar: List { items: Vec::new() },
            content: Vec::new(),
        }
    }

    pub fn parse_markdown(&mut self) {
        while self.pos < self.doc.len() {
            let block = self.parse_block();
            match block {
                LineBreak => {},
                _ => { self.content.push(block); },
            }
        }
    }

    fn parse_block(&mut self) -> Block {
        let c = self.doc[self.pos];

        // header
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

        // blockquote
        if self.expect("> ") {
            return self.parse_blockquote();
        }

        // list
        if c == '*' || c == '+' || c == '-' {
            return self.parse_list(-1);
        }

        // code block
        if self.expect("```") {
            return self.parse_code_block();
        }

        // paragraph, line break
        let spans = self.parse_spans();
        if !spans.is_empty() {
            return Paragraph { spans };
        } else {
            return LineBreak;
        }
    }

    fn parse_header(&mut self, level: u32) -> Block {
        let spans = self.parse_spans();
        if level == 1 {
            self.title = "".to_string();
            for span in &spans {
                match span {
                    Link { title: t, .. } => { self.title.push_str(t); },
                    Emphasis { text, .. } => { self.title.push_str(text); },
                    Code { code } => { self.title.push_str(code); },
                    Text { text } => { self.title.push_str(text); },
                    _ => {},
                }
            }
        } else {
            // let mut cur = &mut self.sidebar;
            // for i in 2..level {
            //     match cur {
            //         List { items } => {
            //             match items[items.len()-1] {
            //                 ListItem { spans, list } => { cur = list; },
            //             }
            //         },
            //         _ => {},
            //     }
            // }
            // match cur {
            //     List { mut items } => {
            //         items.push(ListItem {
            //             spans: spans.clone(),
            //             list: List { items: Vec::new() },
            //         });
            //     },
            //     _ => {},
            // }
            // Convertor::make_toc(&mut self.sidebar, &spans.clone(), level);
            
        }
        if level == 2 {
            match &mut self.sidebar {
                List { items } => {
                    items.push(ListItem {
                        spans: spans.clone(),
                        list: List { items: Vec::new() },
                    });
                },
                _ => {},
            }
        }
        Header { spans, level }
    }

    // fn make_toc(cur_list: &mut Block, spans: &Vec<Span>, level: u32) {
    //     match cur_list {
    //         List { items } => {
    //             if level == 2 {
    //                 items.push(ListItem {
    //                     spans: spans.clone(),
    //                     list: List { items: Vec::new() }
    //                 });
    //             } else {
    //                 match items[items.len()-1] {
    //                     ListItem { ref mut list, .. } => { Convertor::make_toc(list, spans, level - 1); },
    //                 }
                    
    //             }
    //         },
    //         _ => {},
    //     }
    // }

    fn parse_blockquote(&mut self) -> Block {
        Blockquote { spans: self.parse_spans() }
    }

    fn parse_list(&mut self, indent: i32) -> Block {
        let mut items = Vec::new();
        while self.pos < self.doc.len() {
            let mut num = 0;
            while self.pos + num < self.doc.len() {
                let c = self.doc[self.pos + num];
                if c == ' ' {
                    num += 1;
                } else {
                    break;
                }
            }

            let c1 = self.doc[self.pos + num];
            let c2 = self.doc[self.pos + num + 1];
            if (c1 == '*' || c1 == '+' || c1 == '-') && c2 == ' ' {
                if indent < num as i32 {
                    self.pos += num + 2;
                    items.push(ListItem {
                        spans: self.parse_spans(),
                        list: self.parse_list(num as i32),
                    });
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        List { items }
    }

    fn parse_code_block(&mut self) -> Block {
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if self.expect("\n") || self.expect("\r\n") {
                break;
            }
            self.pos += 1;
        }

        let mut code = "".to_string();
        while self.pos < self.doc.len() {
            if self.expect("```") {
                break;
            }
            code.push(self.doc[self.pos]);
            self.pos += 1;
        }

        CodeBlock { code }
    }

    fn parse_paragraph(&mut self) -> Block {
        Paragraph { spans: self.parse_spans() }
    }

    fn parse_spans(&mut self) -> Vec<Span> {
        let mut spans = Vec::new();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if self.expect("\n") || self.expect("\r\n") { // ends at new line
                break;
            }

            // link
            if c == '[' {
                spans.push(self.parse_link());
                continue;
            }

            // emphasis
            if self.expect("**") {
                spans.push(self.parse_emphasis("**"));
                continue;
            }
            if self.expect("__") {
                spans.push(self.parse_emphasis("__"));
                continue;
            }
            if self.expect("*") {
                spans.push(self.parse_emphasis("*"));
                continue;
            }
            if self.expect("_") {
                spans.push(self.parse_emphasis("_"));
                continue;
            }

            // code
            if self.expect("`") {
                spans.push(self.parse_code());
                continue;
            }

            // text
            spans.push(self.parse_text());
        }
        spans
    }

    fn parse_link(&mut self) -> Span {
        self.consume("[");

        let mut title = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            self.pos += 1;
            if c == ']' {
                break;
            }
            title.push(c);
        }
        
        if self.expect("(") {
            let mut url = "".to_string();
            while self.pos < self.doc.len() {
                let c = self.doc[self.pos];
                self.pos += 1;
                if c == ')' {
                    break;
                }
                url.push(c);
            }
            Link { title, url }
        } else { // exception
            Text { text: title }
        }
    }

    fn parse_emphasis(&mut self, ind: &str) -> Span {
        let mut text = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if self.expect("\n") || self.expect("\r\n") {
                return Text { text: format!("{}{}", ind.to_string(), text) };
            }
            if self.expect(ind) {
                break;
            }
            text.push(c);
            self.pos += 1;
        }
        if ind == "*" || ind == "_" {
            return Emphasis { kind: Em, text };
        } else {
            return Emphasis { kind: Strong, text };
        }
    }

    fn parse_code(&mut self) -> Span {
        let mut code = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            self.pos += 1;
            if c == '`' {
                break;
            }
            code.push(c);
        }
        Code { code }
    }

    fn parse_text(&mut self) -> Span {
        let mut text = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' || c == '\r' {
                break;
            }
            if c == '[' || c == '`' || c == '*' || c == '_' {
                break;
            }
            text.push(c);
            self.pos += 1;
        }
        Text { text }
    }

    fn expect(&mut self, s: &str) -> bool {
        let cs: Vec<char> = s.chars().collect();
        for i in 0..s.len() {
            if self.pos + i < self.doc.len() && self.doc[self.pos + i] != cs[i] {
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

    pub fn gen_html(&self, dest: &mut File) {
        writeln!(dest, "<!DOCTYPE html>");
        writeln!(dest, "<html>");
        writeln!(dest, "<head>");
        writeln!(dest, "  <meta charset=\"utf-8\">");
        writeln!(dest, "  <link rel=\"stylesheet\" href=\"./index.css\">");
        writeln!(dest, "  <title>{}</title>", self.title);
        writeln!(dest, "</head>");
        writeln!(dest, "<body>");
        
        writeln!(dest, "  <div id=\"wrapper\">");

        self.gen_sidebar(dest);
        self.gen_content(dest);

        writeln!(dest, "</body>");
        write!(dest, "</html>");
    }

    fn gen_sidebar(&self, dest: &mut File) {
        writeln!(dest, "    <nav id=\"sidebar\">");
        match &self.sidebar {
            List { items } => { self.gen_list(items, 0, dest); },
            _ => {},
        }
        writeln!(dest, "    </nav>");
    }

    fn gen_content(&self, dest: &mut File) {
        writeln!(dest, "    <div id=\"content\">");
        for block in &self.content {
            match block {
                Header { spans, level } => { self.gen_header(spans, level, dest); },
                Blockquote { spans } => { self.gen_blockquote(spans, dest); },
                List { items } => { self.gen_list(items, 0, dest); },
                Paragraph { spans } => { self.gen_paragraph(spans, dest); },
                CodeBlock { code } => { self.gen_code_block(code, dest); },
                _ => {},
            }
        }
        writeln!(dest, "    <div>");
    }

    fn gen_header(&self, spans: &Vec<Span>, level: &u32, dest: &mut File) {
        write!(dest, "      <h{}>", *level);
        self.gen_spans(spans, dest);
        writeln!(dest, "</h{}>", *level);
    }

    fn gen_blockquote(&self, spans: &Vec<Span>, dest: &mut File) {
        write!(dest, "      <blockquote>");
        self.gen_spans(spans, dest);
        writeln!(dest, "</blockquote>");
    }

    fn gen_list(&self, items: &Vec<ListItem>, indent: u32, dest: &mut File) {
        for i in 0..(3 + indent) { write!(dest, "  "); }
        writeln!(dest, "<ul>");
        for item in items {
            for i in 0..(4 + indent) { write!(dest, "  "); }
            writeln!(dest, "<li>");
            match item {
                ListItem { spans, list } => {
                    for i in 0..(5 + indent) { write!(dest, "  "); }
                    self.gen_spans(spans, dest);
                    writeln!(dest);
                    match list {
                        List { items: nested_items } => {
                            if !nested_items.is_empty() { self.gen_list(nested_items, indent + 2, dest); }
                        },
                        _ => {},
                    }
                }
            }
            for i in 0..(4 + indent) { write!(dest, "  "); }
            writeln!(dest, "</li>");
        }
        for i in 0..(3 + indent) { write!(dest, "  "); }
        writeln!(dest, "</ul>");
    }

    fn gen_code_block(&self, code: &String, dest: &mut File) {
        write!(dest, "      <pre>");
        write!(dest, "{}", code);
        writeln!(dest, "</pre>");
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
                Emphasis { kind, text } => { self.gen_emphasis(kind, text, dest); },
                Code { code } => { self.gen_code(code, dest); },
                Text { text } => { self.gen_text(text, dest); },
                _ => {},
            }
        }
    }

    fn gen_link(&self, title: &String, url: &String, dest: &mut File) {
        write!(dest, "<a href=\"{}\">{}</a>", *url, *title);
    }

    fn gen_emphasis(&self, kind: &EmphasisKind, text: &String, dest: &mut File) {
        match *kind {
            Em => { write!(dest, "<em>{}</em>", *text); },
            Strong => { write!(dest, "<strong>{}</strong>", *text); },
        }
    }

    fn gen_code(&self, code: &String, dest: &mut File) {
        write!(dest, "<code>{}</code>", *code);
    }

    fn gen_text(&self, text: &String, dest: &mut File) {
        write!(dest, "{}", text);
    }
}