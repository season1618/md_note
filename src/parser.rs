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

    fn parse_math_block(&mut self) -> Block {
        let mut math = "".to_string();
        while self.pos < self.doc.len() {
            if self.expect("$$") {
                break;
            }
            math.push(self.doc[self.pos]);
            self.pos += 1;
        }

        MathBlock { math }
    }

    fn parse_code_block(&mut self) -> Block {
        let mut lang = "".to_string();
        while self.pos < self.doc.len() {
            if self.expect("\n") || self.expect("\r\n") {
                break;
            }
            lang.push(self.doc[self.pos]);
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
        self.consume("[");

        let mut title = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' || c == '\r' {
                return Text { text: format!("[{}", title) };
            }
            if self.expect("]") {
                break;
            }
            title.push(c);
            self.pos += 1;
        }
        
        if self.expect("(") {
            let mut url = "".to_string();
            while self.pos < self.doc.len() {
                let c = self.doc[self.pos];
                if c == '\n' || c == '\r' {
                    return Text { text: format!("[{}]({}", title, url) };
                }
                if self.expect(")") {
                    break;
                }
                url.push(c);
                self.pos += 1;
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
            if c == '\n' || c == '\r' {
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

    fn parse_math(&mut self) -> Span {
        let mut math = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' || c == '\r' {
                return Text { text: format!("`{}", math) };
            }
            if self.expect("$") {
                break;
            }
            math.push(c);
            self.pos += 1;
        }
        Math { math }
    }

    fn parse_code(&mut self) -> Span {
        let mut code = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' || c == '\r' {
                return Text { text: format!("`{}", code) };
            }
            if self.expect("`") {
                break;
            }
            code.push(c);
            self.pos += 1;
        }
        Code { code }
    }

    fn parse_image(&mut self) -> Span {
        self.consume("(");
        let mut url = "".to_string();
        while self.pos < self.doc.len() {
            let c = self.doc[self.pos];
            if c == '\n' || c == '\r' {
                return Text { text: format!("![]({}", url) };
            }
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
        for i in 0..s.len() {
            if self.pos + i >= self.doc.len() || self.doc[self.pos + i] != cs[i] {
                return false;
            }
        }
        self.pos += s.len();
        return true;
    }

    fn consume(&mut self, s: &str) {
        let cs: Vec<char> = s.chars().collect();
        for i in 0..s.len() {
            if self.pos + i >= self.doc.len() || self.doc[self.pos + i] != cs[i] {
                panic!("syntax error");
            }
        }
        self.pos += s.len();
    }
}