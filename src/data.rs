#[derive(Debug)]
pub enum Block {
    Header { spans: Vec<Span>, level: u32, id: String },
    Blockquote { spans: Vec<Span> },
    ListElement(List),
    CodeBlock { code: String },
    Table { head: Vec<Vec<String>>, body: Vec<Vec<String>> },
    Paragraph { spans: Vec<Span> },
    LineBreak,
}

#[derive(Clone, Debug)]
pub enum Span {
    Link { title: String, url: String },
    Emphasis { kind: EmphasisKind, text: String },
    Code { code: String },
    Image { url: String },
    Text { text: String },
}

#[derive(Debug)]
pub struct List {
    pub ordered: bool,
    pub items: Vec<ListItem>,
}

#[derive(Debug)]
pub struct ListItem {
    pub spans: Vec<Span>,
    pub list: List,
}

#[derive(Clone, Debug)]
pub enum EmphasisKind {
    Em,
    Strong,
}