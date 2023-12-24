use std::io;
use std::io::Write;
use std::fs::File;

use crate::data::*;
use crate::template::*;

use Block::*;
use Span::*;
use EmphasisKind::*;
use Elem::*;

pub fn gen_html(dest: &mut File, title: &String, toc: &List, content: &Vec<Block>, template: &Vec<Elem>) -> Result<(), io::Error> {
    for line in template {
        match line {
            Title(ref indent) => { writeln!(dest, "{:>indent$}<title>{}</title>", " ", title)?; },
            Toc(indent) => { gen_sidebar(dest, title, toc, *indent)?; },
            Content(indent) => { gen_content(dest, content, *indent)?; },
            Str(text) => { write!(dest, "{}", text)?; },
        }
    }
    Ok(())
}

fn gen_sidebar(dest: &mut File, title: &String, toc: &List, indent: usize) -> Result<(), io::Error> {
    writeln!(dest, "{:>indent$}<nav id=\"toc\">", " ")?;
    writeln!(dest, "{:>indent$}  <h4>{}</h4>", " ", title)?;
    gen_list(&toc, indent + 2, dest)?;
    writeln!(dest, "{:>indent$}</nav>", " ")
}

fn gen_content(dest: &mut File, content: &Vec<Block>, indent: usize) -> Result<(), io::Error> {
    writeln!(dest, "{:>indent$}<div id=\"content\">", " ")?;
    for block in content {
        match block {
            Header { spans, level, id } => { gen_header(spans, level, id, indent + 2, dest)?; },
            Blockquote { spans } => { gen_blockquote(spans, indent + 2, dest)?; },
            ListElement(list) => { gen_list(list, indent + 2, dest)?; },
            Table { head, body } => { gen_table(head, body, indent + 2, dest)?; },
            Paragraph { spans } => { gen_paragraph(spans, indent + 2, dest)?; },
            CodeBlock { lang, code } => { gen_code_block(lang, code, indent + 2, dest)?; },
            _ => {},
        }
    }
    writeln!(dest, "{:>indent$}</div>", " ")
}

fn gen_header(spans: &Vec<Span>, level: &u32, id: &String, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "{:>indent$}<h{} id=\"{}\">", " ", *level, *id)?;
    gen_spans(spans, dest)?;
    writeln!(dest, "</h{}>", *level)
}

fn gen_blockquote(spans: &Vec<Span>, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "{:>indent$}<blockquote>", " ")?;
    gen_spans(spans, dest)?;
    writeln!(dest, "</blockquote>")
}

fn gen_list(list: &List, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    if list.items.is_empty() {
        return Ok(());
    }

    writeln!(dest, "{:>indent$}<{}>", " ", if list.ordered { "ol" } else { "ul" })?;
    for item in &list.items {
        writeln!(dest, "{:>indent$}  <li>", " ")?;
        
        write!(dest, "{:>indent$}    ", " ")?;
        gen_spans(&item.spans, dest)?;
        writeln!(dest)?;
        gen_list(&item.list, indent + 4, dest)?;
        
        writeln!(dest, "{:>indent$}  </li>", " ")?;
    }
    writeln!(dest, "{:>indent$}</{}>", " ", if list.ordered { "ol" } else { "ul" })
}

fn gen_table(head: &Vec<Vec<String>>, body: &Vec<Vec<String>>, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    writeln!(dest, "{:>indent$}<table>", " ")?;

    writeln!(dest, "{:>indent$}  <thead>", " ")?;
    for row in head {
        writeln!(dest, "{:>indent$}    <tr>", " ")?;
        for data in row {
            writeln!(dest, "{:>indent$}      <td>{}</td>", " ", *data)?;
        }
        writeln!(dest, "{:>indent$}    </tr>", " ")?;
    }
    writeln!(dest, "{:>indent$}  </thead>", " ")?;
    
    writeln!(dest, "{:>indent$}  <tbody>", " ")?;
    for row in body {
        writeln!(dest, "{:>indent$}    <tr>", " ")?;
        for data in row {
            writeln!(dest, "{:>indent$}      <td>{}</td>", " ", *data)?;
        }
        writeln!(dest, "{:>indent$}    </tr>", " ")?;
    }
    writeln!(dest, "{:>indent$}  </tbody>", " ")?;
    
    writeln!(dest, "{:>indent$}</table>", " ")
}

fn gen_code_block(lang: &String, code: &String, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "{:>indent$}<pre><code class=\"language-{}\">", " ", if lang == "" { "plaintext" } else { lang })?;
    write!(dest, "{}", code)?;
    writeln!(dest, "</code></pre>")
}

fn gen_paragraph(spans: &Vec<Span>, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "{:>indent$}<p>", " ")?;
    gen_spans(spans, dest)?;
    writeln!(dest, "</p>")
}

fn gen_spans(spans: &Vec<Span>, dest: &mut File) -> Result<(), io::Error> {
    for span in spans {
        match span {
            Link { title, url } => { gen_link(title, url, dest)?; },
            Emphasis { kind, text } => { gen_emphasis(kind, text, dest)?; },
            Code { code } => { gen_code(code, dest)?; },
            Image { url } => { gen_image(url, dest)?; },
            Text { text } => { gen_text(text, dest)?; },
        }
    }
    Ok(())
}

fn gen_link(title: &String, url: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "<a href=\"{}\">{}</a>", *url, *title)
}

fn gen_emphasis(kind: &EmphasisKind, text: &String, dest: &mut File) -> Result<(), io::Error> {
    match *kind {
        Em => { write!(dest, "<em>{}</em>", *text) },
        Strong => { write!(dest, "<strong>{}</strong>", *text) },
    }
}

fn gen_code(code: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "<code>{}</code>", *code)
}

fn gen_image(url: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "<img src=\"{}\">", *url)
}

fn gen_text(text: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "{}", text)
}