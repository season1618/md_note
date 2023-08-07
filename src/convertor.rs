use std::io::Write;
use std::fs::File;

use Block::*;
use Span::*;
use EmphasisKind::*;

#[derive(Debug)]
enum Block {
    Header { spans: Vec<Span>, level: u32, id: String },
    Blockquote { spans: Vec<Span> },
    ListElement(List),
    CodeBlock { code: String },
    Table { head: Vec<Vec<String>>, body: Vec<Vec<String>> },
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
struct List {
    items: Vec<ListItem>,
}

#[derive(Debug)]
struct ListItem {
    spans: Vec<Span>,
    list: List,
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
    toc: List,
    content: Vec<Block>,
}

impl Convertor {
    pub fn new(doc: String) -> Self {
        Convertor {
            doc: doc.chars().collect(),
            pos: 0,
            title: "".to_string(),
            toc: List { items: Vec::new() },
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
            return ListElement(self.parse_list(-1));
        }

        // table
        if c == '|' {
            return self.parse_table();
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
        let mut id = "".to_string();
        for span in &spans {
            match span {
                Link { title, .. } => { id.push_str(title); },
                Emphasis { text, .. } => { id.push_str(text); },
                Code { code } => { id.push_str(code); },
                Text { text } => { id.push_str(text); },
                _ => {},
            }
        }
        if level == 1 {
            self.title = id.clone();
        } else {
            let url = format!("#{}", id);

            let mut cur = &mut self.toc;
            for _ in 2..level {
                cur = &mut cur.items.last_mut().unwrap().list;
            }
            cur.items.push(ListItem {
                spans: vec![ Link { title: id.clone(), url }],
                list: List { items: Vec::new() },
            });
            
            // match &mut self.sidebar {
            //     List { items } => {
            //         items.push(ListItem {
            //             spans: vec![ Link { title: id.clone(), url }],
            //             list: List { items: Vec::new() },
            //         });
            //     },
            //     _ => {},
            // }

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
        Header { spans, level, id }
    }

    fn parse_blockquote(&mut self) -> Block {
        Blockquote { spans: self.parse_spans() }
    }

    fn parse_list(&mut self, indent: i32) -> List {
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

    fn parse_table(&mut self) -> Block {
        let mut head = Vec::new();
        let mut body = Vec::new();
        loop {
            let mut row = Vec::new();
            let mut data = "".to_string();
            let mut is_row = false;
            if !self.expect("|") {
                break;
            }
            while self.pos < self.doc.len() {
                if self.expect("\n") || self.expect("\r\n") {
                    break;
                }
                if self.expect("|") {
                    row.push(data);
                    data = "".to_string();
                } else {
                    if self.doc[self.pos] != ' ' && self.doc[self.pos] != '-' {
                        is_row = true;
                    }
                    data.push(self.doc[self.pos]);
                    self.pos += 1;
                }
            }
            if is_row {
                head.push(row);
            } else {
                break;
            }
        }

        loop {
            let mut row = Vec::new();
            let mut data = "".to_string();
            let mut is_row = false;
            if !self.expect("|") {
                break;
            }
            while self.pos < self.doc.len() {
                if self.expect("\n") || self.expect("\r\n") {
                    break;
                }
                if self.expect("|") {
                    row.push(data);
                    data = "".to_string();
                } else {
                    if self.doc[self.pos] != ' ' && self.doc[self.pos] != '-' {
                        is_row = true;
                    }
                    data.push(self.doc[self.pos]);
                    self.pos += 1;
                }
            }
            if is_row {
                body.push(row);
            } else {
                break;
            }
        }

        Table { head, body }
    }

    fn parse_code_block(&mut self) -> Block {
        while self.pos < self.doc.len() {
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

            // image
            if self.expect("![]") {
                spans.push(self.parse_image());
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

    fn parse_image(&mut self) -> Span {
        self.consume("(");
        let mut url = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if self.expect(")") {
                break;
            }
            url.push(c);
            self.pos += 1;
        }
        Image { url }
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
        writeln!(dest, "<!DOCTYPE html>").unwrap();
        writeln!(dest, "<html>").unwrap();
        writeln!(dest, "<head>").unwrap();
        writeln!(dest, "  <meta charset=\"utf-8\">").unwrap();
        writeln!(dest, "  <link rel=\"stylesheet\" href=\"./index.css\">").unwrap();
        writeln!(dest, "  <title>{}</title>", self.title).unwrap();
        writeln!(dest, "</head>").unwrap();
        writeln!(dest, "<body>").unwrap();
        
        writeln!(dest, "  <div id=\"wrapper\">").unwrap();

        self.gen_sidebar(dest);
        self.gen_content(dest);

        writeln!(dest, "</body>").unwrap();
        write!(dest, "</html>").unwrap();
    }

    fn gen_sidebar(&self, dest: &mut File) {
        writeln!(dest, "    <nav id=\"sidebar\">").unwrap();
        self.gen_ordered_list(&self.toc, 0, dest);
        writeln!(dest, "    </nav>").unwrap();
    }

    fn gen_content(&self, dest: &mut File) {
        writeln!(dest, "    <div id=\"content\">").unwrap();
        for block in &self.content {
            match block {
                Header { spans, level, id } => { self.gen_header(spans, level, id, dest); },
                Blockquote { spans } => { self.gen_blockquote(spans, dest); },
                ListElement(list) => { self.gen_unordered_list(list, 0, dest); },
                Table { head, body } => { self.gen_table(head, body, dest); },
                Paragraph { spans } => { self.gen_paragraph(spans, dest); },
                CodeBlock { code } => { self.gen_code_block(code, dest); },
                _ => {},
            }
        }
        writeln!(dest, "    <div>").unwrap();
    }

    fn gen_header(&self, spans: &Vec<Span>, level: &u32, id: &String, dest: &mut File) {
        write!(dest, "      <h{} id=\"{}\">", *level, *id).unwrap();
        self.gen_spans(spans, dest);
        writeln!(dest, "</h{}>", *level).unwrap();
    }

    fn gen_blockquote(&self, spans: &Vec<Span>, dest: &mut File) {
        write!(dest, "      <blockquote>").unwrap();
        self.gen_spans(spans, dest);
        writeln!(dest, "</blockquote>").unwrap();
    }

    fn gen_unordered_list(&self, list: &List, indent: u32, dest: &mut File) {
        if list.items.is_empty() {
            return;
        }

        for _ in 0..(3 + indent) { write!(dest, "  ").unwrap(); }
        writeln!(dest, "<ul>").unwrap();
        for item in &list.items {
            for _ in 0..(4 + indent) { write!(dest, "  ").unwrap(); }
            writeln!(dest, "<li>").unwrap();
            
            for _ in 0..(5 + indent) { write!(dest, "  ").unwrap(); }
            self.gen_spans(&item.spans, dest);
            writeln!(dest).unwrap();
            self.gen_unordered_list(&item.list, indent + 2, dest);
            
            for _ in 0..(4 + indent) { write!(dest, "  ").unwrap(); }
            writeln!(dest, "</li>").unwrap();
        }
        for _ in 0..(3 + indent) { write!(dest, "  ").unwrap(); }
        writeln!(dest, "</ul>").unwrap();
    }

    fn gen_ordered_list(&self, list: &List, indent: u32, dest: &mut File) {
        if list.items.is_empty() {
            return;
        }

        for _ in 0..(3 + indent) { write!(dest, "  ").unwrap(); }
        writeln!(dest, "<ol>").unwrap();
        for item in &list.items {
            for _ in 0..(4 + indent) { write!(dest, "  ").unwrap(); }
            writeln!(dest, "<li>").unwrap();
            
            for _ in 0..(5 + indent) { write!(dest, "  ").unwrap(); }
            self.gen_spans(&item.spans, dest);
            writeln!(dest).unwrap();
            self.gen_ordered_list(&item.list, indent + 2, dest);
            
            for _ in 0..(4 + indent) { write!(dest, "  ").unwrap(); }
            writeln!(dest, "</li>").unwrap();
        }
        for _ in 0..(3 + indent) { write!(dest, "  ").unwrap(); }
        writeln!(dest, "</ol>").unwrap();
    }

    fn gen_table(&self, head: &Vec<Vec<String>>, body: &Vec<Vec<String>>, dest: &mut File) {
        writeln!(dest, "      <table>").unwrap();

        writeln!(dest, "        <thead>").unwrap();
        for row in head {
            writeln!(dest, "          <tr>").unwrap();
            for data in row {
                writeln!(dest, "            <td>{}</td>", *data).unwrap();
            }
            writeln!(dest, "          </tr>").unwrap();
        }
        writeln!(dest, "        </thead>").unwrap();
        
        writeln!(dest, "        <tbody>").unwrap();
        for row in body {
            writeln!(dest, "          <tr>").unwrap();
            for data in row {
                writeln!(dest, "            <td>{}</td>", *data).unwrap();
            }
            writeln!(dest, "          </tr>").unwrap();
        }
        writeln!(dest, "        </tbody>").unwrap();
        
        writeln!(dest, "      </table>").unwrap();
    }

    fn gen_code_block(&self, code: &String, dest: &mut File) {
        write!(dest, "      <pre>").unwrap();
        write!(dest, "{}", code).unwrap();
        writeln!(dest, "</pre>").unwrap();
    }

    fn gen_paragraph(&self, spans: &Vec<Span>, dest: &mut File) {
        write!(dest, "      <p>").unwrap();
        self.gen_spans(spans, dest);
        writeln!(dest, "</p>").unwrap();
    }

    fn gen_spans(&self, spans: &Vec<Span>, dest: &mut File) {
        for span in spans {
            match span {
                Link { title, url } => { self.gen_link(title, url, dest); },
                Emphasis { kind, text } => { self.gen_emphasis(kind, text, dest); },
                Code { code } => { self.gen_code(code, dest); },
                Image { url } => { self.gen_image(url, dest); },
                Text { text } => { self.gen_text(text, dest); },
            }
        }
    }

    fn gen_link(&self, title: &String, url: &String, dest: &mut File) {
        write!(dest, "<a href=\"{}\">{}</a>", *url, *title).unwrap();
    }

    fn gen_emphasis(&self, kind: &EmphasisKind, text: &String, dest: &mut File) {
        match *kind {
            Em => { write!(dest, "<em>{}</em>", *text).unwrap(); },
            Strong => { write!(dest, "<strong>{}</strong>", *text).unwrap(); },
        }
    }

    fn gen_code(&self, code: &String, dest: &mut File) {
        write!(dest, "<code>{}</code>", *code).unwrap();
    }

    fn gen_image(&self, url: &String, dest: &mut File) {
        write!(dest, "<img src=\"{}\">", *url).unwrap();
    }

    fn gen_text(&self, text: &String, dest: &mut File) {
        write!(dest, "{}", text).unwrap();
    }
}