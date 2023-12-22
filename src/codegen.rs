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
            Title(indent) => { writeln!(dest, "  <title>{}</title>", title)?; },
            Toc(indent) => { gen_sidebar(dest, title, toc)?; },
            Content(indent) => { gen_content(dest, content)?; },
            Str(text) => { write!(dest, "{}", text)?; },
        }
    }
    Ok(())
}

fn gen_sidebar(dest: &mut File, title: &String, toc: &List) -> Result<(), io::Error> {
    writeln!(dest, "    <nav id=\"toc\">")?;
    writeln!(dest, "      <h4>{}</h4>", title)?;
    gen_list(&toc, 6, dest)?;
    writeln!(dest, "    </nav>")
}

fn gen_content(dest: &mut File, content: &Vec<Block>) -> Result<(), io::Error> {
    writeln!(dest, "    <div id=\"content\">")?;
    for block in content {
        match block {
            Header { spans, level, id } => { gen_header(spans, level, id, dest)?; },
            Blockquote { spans } => { gen_blockquote(spans, dest)?; },
            ListElement(list) => { gen_list(list, 6, dest)?; },
            Table { head, body } => { gen_table(head, body, dest)?; },
            Paragraph { spans } => { gen_paragraph(spans, dest)?; },
            CodeBlock { code } => { gen_code_block(code, dest)?; },
            _ => {},
        }
    }
    writeln!(dest, "    <div>")
}

fn gen_header(spans: &Vec<Span>, level: &u32, id: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "      <h{} id=\"{}\">", *level, *id)?;
    gen_spans(spans, dest)?;
    writeln!(dest, "</h{}>", *level)
}

fn gen_blockquote(spans: &Vec<Span>, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "      <blockquote>")?;
    gen_spans(spans, dest)?;
    writeln!(dest, "</blockquote>")
}

fn gen_list(list: &List, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    if list.items.is_empty() {
        return Ok(());
    }

    writeln!(dest, "{}<{}>", " ".repeat(indent), if list.ordered { "ol" } else { "ul" })?;
    for item in &list.items {
        writeln!(dest, "{}<li>", " ".repeat(indent + 2))?;
        
        write!(dest, "{}", " ".repeat(indent + 4))?;
        gen_spans(&item.spans, dest)?;
        writeln!(dest)?;
        gen_list(&item.list, indent + 4, dest)?;
        
        writeln!(dest, "{}</li>", " ".repeat(indent + 2))?;
    }
    writeln!(dest, "{}</{}>", " ".repeat(indent), if list.ordered { "ol" } else { "ul" })
}

fn gen_table(head: &Vec<Vec<String>>, body: &Vec<Vec<String>>, dest: &mut File) -> Result<(), io::Error> {
    writeln!(dest, "      <table>")?;

    writeln!(dest, "        <thead>")?;
    for row in head {
        writeln!(dest, "          <tr>")?;
        for data in row {
            writeln!(dest, "            <td>{}</td>", *data)?;
        }
        writeln!(dest, "          </tr>")?;
    }
    writeln!(dest, "        </thead>")?;
    
    writeln!(dest, "        <tbody>")?;
    for row in body {
        writeln!(dest, "          <tr>")?;
        for data in row {
            writeln!(dest, "            <td>{}</td>", *data)?;
        }
        writeln!(dest, "          </tr>")?;
    }
    writeln!(dest, "        </tbody>")?;
    
    writeln!(dest, "      </table>")
}

fn gen_code_block(code: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "      <pre>")?;
    write!(dest, "{}", code)?;
    writeln!(dest, "</pre>")
}

fn gen_paragraph(spans: &Vec<Span>, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "      <p>")?;
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