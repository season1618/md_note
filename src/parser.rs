use tokio;
use regex::Regex;
use reqwest::{self, header};

use crate::data::*;
use crate::multiset::MultiSet;
use Block::*;
use Span::*;

pub fn parse_markdown(doc: &String) -> (String, List, Vec<Block>) {
    let mut parser = Parser::new(doc);
    parser.parse_markdown();
    return (parser.title, parser.toc, parser.content);
}

pub struct Parser {
    doc: Vec<char>,
    pos: usize,
    headers: MultiSet<String>,
    title: String,
    toc: List,
    content: Vec<Block>,
}

impl Parser {
    fn new(doc: &String) -> Self {
        Parser {
            doc: doc.chars().collect(),
            pos: 0,
            headers: MultiSet::new(),
            title: String::new(),
            toc: List { ordered: true, items: Vec::new() },
            content: Vec::new(),
        }
    }

    pub fn parse_markdown(&mut self) {
        while self.pos < self.doc.len() {
            let block = self.parse_block();
            match block {
                Paragraph { spans } if spans.is_empty() => {},
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
        if c == '*' || c == '+' || c == '-' || self.starts_with_number_period_space() {
            return ListElement(self.parse_list(0));
        }

        // link card
        if self.expect("?[](") {
            return self.parse_link_card();
        }

        // table
        if c == '|' {
            return self.parse_table();
        }

        // math block
        if self.expect("$$") {
            return self.parse_math_block();
        }

        // code block
        if self.expect("```") {
            return self.parse_code_block();
        }

        // paragraph
        return self.parse_paragraph();
    }

    fn parse_header(&mut self, level: u32) -> Block {
        let spans = self.parse_spans();
        let mut header = "".to_string();
        for span in &spans {
            match span {
                Link { text, .. } => { header.push_str(text); },
                Emphasis { text, .. } => { header.push_str(text); },
                Math { math } => { header.push_str(&format!("\\({}\\)", math)) },
                Code { code } => { header.push_str(code); },
                Text { text } => { header.push_str(text); },
                _ => {},
            }
        }

        let count = self.headers.insert(header.clone());
        let id = if count == 0 { format!("{}", &header) } else { format!("{}-{}", &header, count) };
        let href = format!("#{}", &id);

        // modify title or table of contents
        if level == 1 {
            self.title = header.clone();
        } else {
            let mut cur = &mut self.toc;
            for _ in 2..level {
                cur = &mut cur.items.last_mut().unwrap().list;
            }
            cur.items.push(ListItem {
                spans: vec![ Link { text: header.clone(), url: href.clone() }],
                list: List { ordered: true, items: Vec::new() },
            });
        }
        Header { spans, level, id }
    }

    fn parse_blockquote(&mut self) -> Block {
        Blockquote { spans: self.parse_spans() }
    }

    fn parse_list(&mut self, min_indent: usize) -> List {
        let mut ordered = false;
        let mut items = Vec::new();
        while self.pos < self.doc.len() {
            let mut indent = 0;
            while self.pos + indent < self.doc.len() && self.doc[self.pos + indent] == ' ' {
                indent += 1;
            }

            if min_indent <= indent {
                self.pos += indent;

                let c1 = self.doc[self.pos];
                let c2 = self.doc[self.pos + 1];
                if (c1 == '*' || c1 == '+' || c1 == '-') && c2 == ' ' {
                    self.pos += 2;
                    ordered = false;
                    items.push(ListItem {
                        spans: self.parse_spans(),
                        list: self.parse_list(indent + 1),
                    });
                    continue;
                }

                if self.expect_number_period_space() {
                    ordered = true;
                    items.push(ListItem {
                        spans: self.parse_spans(),
                        list: self.parse_list(indent + 1),
                    });
                    continue;
                }
            }
            break;
        }
        List { ordered, items }
    }

    fn parse_link_card(&mut self) -> Block {
        let mut url = "".to_string();
        while let Some(c) = self.next_char_term(")") {
            url.push(c);
        }
        let (title, image, description, site_name) = get_ogp_info(&url);
        LinkCard { title, image, url, description, site_name }
    }

    fn parse_table(&mut self) -> Block {
        let mut head = Vec::new();
        let mut body = Vec::new();
        while let Some(row) = self.parse_table_row() {
            head.push(row);
        }
        while let Some(row) = self.parse_table_row() {
            body.push(row);
        }
        Table { head, body }
    }

    fn parse_table_row(&mut self) -> Option<Vec<String>> {
        if !self.expect("|") {
            return None;
        }

        let mut row: Vec<String> = Vec::new();
        loop {
            let mut data = "".to_string();
            loop {
                if self.expect("\n") || self.expect("\r\n") {
                    if row.iter().all(|s| s.chars().all(|c| c == '-' || c == ' ')) {
                        return None;
                    }
                    return Some(row);
                }
                if self.expect("|") {
                    row.push(data);
                    break;
                }
                if let Some(c) = self.next_char() {
                    data.push_str(&self.escape(c));
                    continue;
                }
                return Some(row);
            }
        }
    }

    fn parse_math_block(&mut self) -> Block {
        let mut math = "".to_string();
        while let Some(c) = self.next_char_term("$$") {
            math.push_str(&self.escape(c));
        }
        MathBlock { math }
    }

    fn parse_code_block(&mut self) -> Block {
        let mut lang = "".to_string();
        while let Some(c) = self.next_char_term("\n") {
            lang.push(c);
        }
        let mut code = "".to_string();
        while let Some(c) = self.next_char_term("```") {
            code.push_str(&self.escape(c));
        }
        CodeBlock { lang, code }
    }

    fn parse_paragraph(&mut self) -> Block {
        Paragraph { spans: self.parse_spans() }
    }

    fn parse_spans(&mut self) -> Vec<Span> {
        let mut spans = Vec::new();
        while self.pos < self.doc.len() {
            if self.expect("\n") || self.expect("\r\n") { // ends at new line
                break;
            }

            // link
            if self.expect("[") {
                spans.push(self.parse_link());
                continue;
            }

            // strong
            if self.expect("**") {
                spans.push(self.parse_strong('*'));
                continue;
            }
            if self.expect("__") {
                spans.push(self.parse_strong('_'));
                continue;
            }

            // emphasis
            if self.expect("*") {
                spans.push(self.parse_emphasis('*'));
                continue;
            }
            if self.expect("_") {
                spans.push(self.parse_emphasis('_'));
                continue;
            }

            // math
            if self.expect("$") {
                spans.push(self.parse_math());
                continue;
            }

            // code
            if self.expect("`") {
                spans.push(self.parse_code());
                continue;
            }

            // image
            if self.expect("![](") {
                spans.push(self.parse_image());
                continue;
            }

            // text
            spans.push(self.parse_text());
        }
        spans
    }

    fn parse_link(&mut self) -> Span {
        let mut text = "".to_string();
        while let Some(c) = self.next_char_line_term("]") {
            if c == '\n' {
                return Text { text: format!("[{}", text) };
            }
            text.push_str(&self.escape(c));
        }

        if !self.expect("(") {
            return Text { text };
        }

        let mut url = "".to_string();
        while let Some(c) = self.next_char_line_term(")") {
            if c == '\n' {
                return Text { text: format!("[{}]({}", text, url) };
            }
            url.push(c);
        }

        if text.is_empty() {
            text = get_title(&url);
        }

        Link { text, url }
    }

    fn parse_strong(&mut self, d: char) -> Span {
        let mut text = String::new();
        let mut i = 0;
        while let Some(c) = self.next_char_line(i) {
            if c == d {
                if Some(d) == self.next_char_line(i + 1) {
                    self.pos += i + 2;
                    return Strong { text };
                } else {
                    self.pos += i + 1;
                    return Emphasis { text };
                }
            }
            i += 1;
            text.push_str(&self.escape(c));
        }
        Text { text: format!("{0}{0}", d) }
    }

    fn parse_emphasis(&mut self, d: char) -> Span {
        let mut text = String::new();
        let mut i = 0;
        while let Some(c) = self.next_char_line(i) {
            if c == d {
                self.pos += i + 1;
                return Emphasis { text };
            }
            i += 1;
            text.push_str(&self.escape(c));
        }
        Text { text: d.to_string() }
    }

    fn parse_math(&mut self) -> Span {
        let mut math = String::new();
        let mut i = 0;
        while let Some(c) = self.next_char_line(i) {
            if c == '$' {
                self.pos += i + 1;
                return Math { math };
            }
            i += 1;
            math.push_str(&self.escape(c));
        }
        Text { text: "$".to_string() }
    }

    fn parse_code(&mut self) -> Span {
        let mut code = String::new();
        let mut i = 0;
        while let Some(c) = self.next_char_line(i) {
            if c == '`' {
                self.pos += i + 1;
                return Code { code };
            }
            i += 1;
            code.push_str(&self.escape(c));
        }
        Text { text: "`".to_string() }
    }

    fn parse_image(&mut self) -> Span {
        let mut url = String::new();
        let mut i = 0;
        while let Some(c) = self.next_char_line(i) {
            if c == ')' {
                self.pos += i + 1;
                return Image { url };
            }
            i += 1;
            url.push(c);
        }
        Text { text: "![](".to_string() }
    }

    fn parse_text(&mut self) -> Span {
        let mut text = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' || c == '\r' {
                break;
            }
            if c == '[' || c == '$' || c == '`' || c == '*' || c == '_' {
                break;
            }
            self.pos += 1;
            text.push_str(&self.escape(c));
        }
        Text { text }
    }

    fn starts_with_number_period_space(&self) -> bool {
        let chs = &self.doc[self.pos ..];
        let mut i = 0;
        for c in chs {
            if !c.is_ascii_digit() { break; }
            i += 1;
        }
        i > 0 && chs[i..].starts_with(&['.', ' '])
    }

    fn expect_number_period_space(&mut self) -> bool {
        let chs = &self.doc[self.pos ..];
        let mut i = 0;
        for c in chs {
            if !c.is_ascii_digit() { break; }
            i += 1;
        }
        if i > 0 && chs[i..].starts_with(&['.', ' ']) {
            self.pos += i + 2;
            return true;
        }
        false
    }

    fn expect(&mut self, s: &str) -> bool {
        let cs: Vec<char> = s.chars().collect();
        if self.doc[self.pos ..].starts_with(&cs) {
            self.pos += cs.len();
            return true;
        }
        false
    }

    fn next_char(&mut self) -> Option<char> {
        if self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            self.pos += 1;
            return Some(c);
        }
        None
    }

    fn next_char_term(&mut self, term: &str) -> Option<char> {
        let terms: Vec<char> = term.chars().collect();
        if self.doc[self.pos ..].starts_with(&terms) {
            self.pos += terms.len();
            return None;
        }
        if self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            self.pos += 1;
            return Some(c);
        }
        None
    }

    fn next_char_line(&mut self, index: usize) -> Option<char> {
        let chs = &self.doc[self.pos + index..];
        if chs.starts_with(&['\n']) || chs.starts_with(&['\r', '\n']) {
            return None;
        }
        Some(chs[0])
    }

    fn next_char_line_term(&mut self, term: &str) -> Option<char> {
        let terms: Vec<char> = term.chars().collect();
        if self.doc[self.pos ..].starts_with(&terms) {
            self.pos += terms.len();
            return None;
        }
        if self.pos < self.doc.len() {
            let mut c = self.doc[self.pos];
            if c == '\r' {
                self.pos += 1;
                c = self.doc[self.pos];
            }
            if c != '\n' {
                self.pos += 1;
            }
            return Some(c);
        }
        None
    }

    fn escape(&self, c: char) -> String {
        match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            _ => c.to_string(),
        }
    }
}

#[tokio::main]
async fn get_title(url: &String) -> String {
    let client = reqwest::Client::new();
    let Ok(res) = client.get(url).header(header::ACCEPT, header::HeaderValue::from_str("text/html").unwrap()).send().await else {
        return "".to_string();
    };
    let Ok(body) = res.text().await else {
        return "".to_string();
    };
    let regex = Regex::new("<title>(.*)</title>").unwrap();
    if let Some(caps) = regex.captures(&body) {
        return caps[1].to_string().clone();
    }
    return "".to_string();
}

#[tokio::main]
async fn get_ogp_info(url: &String) -> (String, Option<String>, Option<String>, Option<String>) {
    let mut title = "".to_string();
    let mut image = None;
    let mut description = None;
    let mut site_name = None;

    let client = reqwest::Client::new();
    let Ok(res) = client.get(url).header(header::ACCEPT, header::HeaderValue::from_str("text/html").unwrap()).send().await else {
        return (title, image, description, site_name);
    };
    let Ok(body) = res.text().await else {
        return (title, image, description, site_name);
    };

    let regex = Regex::new("property=\"og:([^\"]*)\" content=\"([^\"]*)\"").unwrap();
    for caps in regex.captures_iter(&body) {
        match &caps[1] {
            "title" => { title = caps[2].to_string(); },
            "image" => { image = Some(caps[2].to_string()); },
            "description" => { description = Some(caps[2].to_string()); },
            "site_name" => { site_name = Some(caps[2].to_string()); },
            _ => {},
        }
    }

    if title.is_empty() {
        let regex = Regex::new("<title>(.*)</title>").unwrap();
        if let Some(caps) = regex.captures(&body) {
            title = caps[1].to_string();
        }
    }

    (title, image, description, site_name)
}