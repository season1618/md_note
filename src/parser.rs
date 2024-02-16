use tokio;
use regex::Regex;
use reqwest::{self, header};

use crate::data::*;
use crate::multiset::MultiSet;
use Block::*;
use Span::*;

pub fn parse_markdown(doc: &str) -> (String, List, Vec<Block>) {
    let mut parser = Parser::new(doc);
    parser.parse_markdown();
    return (parser.title, parser.toc, parser.content);
}

pub struct Parser<'a> {
    chs: &'a str,
    headers: MultiSet<String>,
    title: String,
    toc: List,
    content: Vec<Block>,
}

impl<'a> Parser<'a> {
    fn new(doc: &'a str) -> Self {
        Parser {
            chs: doc,
            headers: MultiSet::new(),
            title: String::new(),
            toc: List { ordered: true, items: Vec::new() },
            content: Vec::new(),
        }
    }

    pub fn parse_markdown(&mut self) {
        while !self.chs.is_empty() {
            let block = self.parse_block();
            match block {
                Paragraph { spans } if spans.is_empty() => {},
                _ => { self.content.push(block); },
            }
        }
    }

    fn parse_block(&mut self) -> Block {
        // paragraph
        return self.parse_paragraph();
    }

    fn parse_paragraph(&mut self) -> Block {
        Paragraph { spans: self.parse_spans() }
    }

    fn parse_spans(&mut self) -> Vec<Span> {
        let mut spans = Vec::new();
        while !self.chs.is_empty() && !self.next_newline() {
            // text
            spans.push(self.parse_text());
        }
        spans
    }

    fn parse_text(&mut self) -> Span {
        let mut text = String::new();
        while let Some(c) = self.next_char_except("\r\n") {
            text.push_str(&self.escape(c));
        }
        Text { text }
    }

    fn next_char_except(&mut self, chs: &str) -> Option<char> {
        if let Some(c) = self.chs.chars().nth(0) {
            if !chs.contains(c) {
                let i = if let Some((i, _)) = self.chs.char_indices().nth(1) { i } else { self.chs.len() };
                self.chs = &self.chs[i..];
                return Some(c);
            }
        }
        None
    }

    fn next_newline(&mut self) -> bool {
        if self.chs.starts_with("\n") {
            self.chs = &self.chs[1..];
            return true;
        }
        if self.chs.starts_with("\r\n") {
            self.chs = &self.chs[2..];
            return true;
        }
        false
    }

    fn escape(&self, c: char) -> String {
        match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            _ => c.to_string(),
        }
    }
}