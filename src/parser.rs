use crate::data::*;
use Block::*;
use Span::*;
use EmphasisKind::*;

pub fn parse_markdown(doc: &String) -> (String, List, Vec<Block>) {
    let mut parser = Parser::new(doc);
    parser.parse_markdown();
    return (parser.title, parser.toc, parser.content);
}

pub struct Parser {
    doc: Vec<char>,
    pos: usize,
    title: String,
    toc: List,
    content: Vec<Block>,
}

impl Parser {
    fn new(doc: &String) -> Self {
        Parser {
            doc: doc.chars().collect(),
            pos: 0,
            title: "".to_string(),
            toc: List { ordered: true, items: Vec::new() },
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
        if c == '*' || c == '+' || c == '-' || self.match_numbers_period_space() {
            return ListElement(self.parse_list(0));
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

        // modify title or table of contents
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

                if self.expect_numbers_period_space() {
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
            lang.push_str(&self.escape(c));
        }
        let mut code = "".to_string();
        while let Some(c) = self.next_char_term("```") {
            code.push_str(&self.escape(c));
        }
        CodeBlock { lang, code }
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
        self.expect("[");

        let mut title = "".to_string();
        while let Some(c) = self.next_char_line_term("]") {
            if c == '\n' {
                return Text { text: format!("[{}", title) };
            }
            title.push_str(&self.escape(c));
        }

        if !self.expect("(") {
            return Text { text: title };
        }

        let mut url = "".to_string();
        while let Some(c) = self.next_char_line_term(")") {
            if c == '\n' {
                return Text { text: format!("[{}]({}", title, url) };
            }
            url.push_str(&self.escape(c));
        }

        Link { title, url }
    }

    fn parse_emphasis(&mut self, ind: &str) -> Span {
        let mut text = "".to_string();
        while let Some(c) = self.next_char_line_term(ind) {
            if c == '\n' {
                return Text { text: format!("{}{}", ind.to_string(), text) };
            }
            text.push_str(&self.escape(c));
        }

        if ind == "*" || ind == "_" {
            return Emphasis { kind: Em, text };
        } else {
            return Emphasis { kind: Strong, text };
        }
    }

    fn parse_math(&mut self) -> Span {
        let mut math = "".to_string();
        while let Some(c) = self.next_char_line_term("$") {
            if c == '\n' {
                return Text { text: format!("${}", math) };
            }
            math.push_str(&self.escape(c));
        }
        Math { math }
    }

    fn parse_code(&mut self) -> Span {
        let mut code = "".to_string();
        while let Some(c) = self.next_char_line_term("`") {
            if c == '\n' {
                return Text { text: format!("`{}", code) };
            }
            code.push_str(&self.escape(c));
        }
        Code { code }
    }

    fn parse_image(&mut self) -> Span {
        self.expect("(");
        let mut url = "".to_string();
        while let Some(c) = self.next_char_line_term(")") {
            if c == '\n' {
                return Text { text: format!("![]({}", url) };
            }
            url.push_str(&self.escape(c));
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
            text.push_str(&self.escape(c));
            self.pos += 1;
        }
        Text { text }
    }

    fn match_numbers_period_space(&mut self) -> bool {
        let mut i = 0;
        while self.pos + i < self.doc.len() && self.doc[self.pos + i].is_ascii_digit() {
            i += 1;
        }
        if i > 0 && self.doc[self.pos + i] == '.' && self.doc[self.pos + i + 1] == ' ' {
            return true;
        } else {
            return false;
        }
    }

    fn expect_numbers_period_space(&mut self) -> bool {
        let mut i = 0;
        while self.pos + i < self.doc.len() && self.doc[self.pos + i].is_ascii_digit() {
            i += 1;
        }
        if i > 0 && self.doc[self.pos + i] == '.' && self.doc[self.pos + i + 1] == ' ' {
            self.pos += i + 2;
            return true;
        } else {
            return false;
        }
    }

    fn expect(&mut self, s: &str) -> bool {
        let cs: Vec<char> = s.chars().collect();
        if self.pos + cs.len() <= self.doc.len() && self.doc[self.pos .. self.pos + cs.len()] == cs {
            self.pos += cs.len();
            return true;
        }
        false
    }

    fn next_char(&mut self) -> Option<char> {
        if self.pos < self.doc.len() {
            self.pos += 1;
            return Some(self.doc[self.pos - 1]);
        }
        None
    }

    fn next_char_term(&mut self, term: &str) -> Option<char> {
        let terms: Vec<char> = term.chars().collect();
        if self.pos + terms.len() <= self.doc.len() && self.doc[self.pos .. self.pos + terms.len()] == terms {
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

    fn next_char_line_term(&mut self, term: &str) -> Option<char> {
        let terms: Vec<char> = term.chars().collect();
        if self.pos + terms.len() <= self.doc.len() && self.doc[self.pos .. self.pos + terms.len()] == terms {
            self.pos += terms.len();
            return None;
        }
        if self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c != '\n' && c != '\r' {
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