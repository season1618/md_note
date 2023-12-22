use std::io::Write;
use std::fs::File;

use crate::data::*;
use Block::*;
use Span::*;
use EmphasisKind::*;

pub fn gen_html(dest: &mut File, title: &String, toc: &List, content: &Vec<Block>) {
    writeln!(dest, "<!DOCTYPE html>").unwrap();
    writeln!(dest, "<html>").unwrap();
    writeln!(dest, "<head>").unwrap();
    writeln!(dest, "  <meta charset=\"utf-8\">").unwrap();
    writeln!(dest, "  <link rel=\"stylesheet\" href=\"./index.css\">").unwrap();
    writeln!(dest, "  <title>{}</title>", title).unwrap();
    writeln!(dest, "</head>").unwrap();
    writeln!(dest, "<body>").unwrap();
    
    writeln!(dest, "  <div id=\"wrapper\">").unwrap();

    gen_sidebar(dest, title, toc);
    gen_content(dest, content);

    writeln!(dest, "</body>").unwrap();
    write!(dest, "</html>").unwrap();
}

fn gen_sidebar(dest: &mut File, title: &String, toc: &List) {
    writeln!(dest, "    <nav id=\"sidebar\">").unwrap();
    writeln!(dest, "      <h4>{}</h4>", title).unwrap();
    gen_list(&toc, 6, dest);
    writeln!(dest, "    </nav>").unwrap();
}

fn gen_content(dest: &mut File, content: &Vec<Block>) {
    writeln!(dest, "    <div id=\"content\">").unwrap();
    for block in content {
        match block {
            Header { spans, level, id } => { gen_header(spans, level, id, dest); },
            Blockquote { spans } => { gen_blockquote(spans, dest); },
            ListElement(list) => { gen_list(list, 6, dest); },
            Table { head, body } => { gen_table(head, body, dest); },
            Paragraph { spans } => { gen_paragraph(spans, dest); },
            CodeBlock { code } => { gen_code_block(code, dest); },
            _ => {},
        }
    }
    writeln!(dest, "    <div>").unwrap();
}

fn gen_header(spans: &Vec<Span>, level: &u32, id: &String, dest: &mut File) {
    write!(dest, "      <h{} id=\"{}\">", *level, *id).unwrap();
    gen_spans(spans, dest);
    writeln!(dest, "</h{}>", *level).unwrap();
}

fn gen_blockquote(spans: &Vec<Span>, dest: &mut File) {
    write!(dest, "      <blockquote>").unwrap();
    gen_spans(spans, dest);
    writeln!(dest, "</blockquote>").unwrap();
}

fn gen_list(list: &List, indent: usize, dest: &mut File) {
    if list.items.is_empty() {
        return;
    }

    writeln!(dest, "{}<{}>", " ".repeat(indent), if list.ordered { "ol" } else { "ul" }).unwrap();
    for item in &list.items {
        writeln!(dest, "{}<li>", " ".repeat(indent + 2)).unwrap();
        
        write!(dest, "{}", " ".repeat(indent + 4)).unwrap();
        gen_spans(&item.spans, dest);
        writeln!(dest).unwrap();
        gen_list(&item.list, indent + 4, dest);
        
        writeln!(dest, "{}</li>", " ".repeat(indent + 2)).unwrap();
    }
    writeln!(dest, "{}</{}>", " ".repeat(indent), if list.ordered { "ol" } else { "ul" }).unwrap();
}

fn gen_table(head: &Vec<Vec<String>>, body: &Vec<Vec<String>>, dest: &mut File) {
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

fn gen_code_block(code: &String, dest: &mut File) {
    write!(dest, "      <pre>").unwrap();
    write!(dest, "{}", code).unwrap();
    writeln!(dest, "</pre>").unwrap();
}

fn gen_paragraph(spans: &Vec<Span>, dest: &mut File) {
    write!(dest, "      <p>").unwrap();
    gen_spans(spans, dest);
    writeln!(dest, "</p>").unwrap();
}

fn gen_spans(spans: &Vec<Span>, dest: &mut File) {
    for span in spans {
        match span {
            Link { title, url } => { gen_link(title, url, dest); },
            Emphasis { kind, text } => { gen_emphasis(kind, text, dest); },
            Code { code } => { gen_code(code, dest); },
            Image { url } => { gen_image(url, dest); },
            Text { text } => { gen_text(text, dest); },
        }
    }
}

fn gen_link(title: &String, url: &String, dest: &mut File) {
    write!(dest, "<a href=\"{}\">{}</a>", *url, *title).unwrap();
}

fn gen_emphasis(kind: &EmphasisKind, text: &String, dest: &mut File) {
    match *kind {
        Em => { write!(dest, "<em>{}</em>", *text).unwrap(); },
        Strong => { write!(dest, "<strong>{}</strong>", *text).unwrap(); },
    }
}

fn gen_code(code: &String, dest: &mut File) {
    write!(dest, "<code>{}</code>", *code).unwrap();
}

fn gen_image(url: &String, dest: &mut File) {
    write!(dest, "<img src=\"{}\">", *url).unwrap();
}

fn gen_text(text: &String, dest: &mut File) {
    write!(dest, "{}", text).unwrap();
}