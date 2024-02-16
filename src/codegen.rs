use std::io::{self, Write};
use std::fs::File;
use chrono::{Local, Datelike, Timelike};

use crate::data::*;

use Block::*;
use Span::*;
use Elem::*;

pub fn gen_html(dest: &mut File, title: &String, toc: &List, content: &Vec<Block>, template: &Vec<Elem>) -> Result<(), io::Error> {
    let datetime = Local::now();
    for chunk in template {
        match chunk {
            Title => { write!(dest, "{}", title)?; },
            Year => { write!(dest, "{:04}", datetime.year())?; },
            Month => { write!(dest, "{:02}", datetime.month())?; },
            Day => { write!(dest, "{:02}", datetime.day())?; },
            Hour => { write!(dest, "{:02}", datetime.hour())?; },
            Minute => { write!(dest, "{:02}", datetime.minute())?; },
            Second => { write!(dest, "{:02}", datetime.second())?; },
            Toc(indent) => { gen_toc(dest, toc, *indent)?; },
            Content(indent) => { gen_content(dest, content, *indent)?; },
            Str(text) => { write!(dest, "{}", text)?; },
        }
    }
    Ok(())
}

fn gen_toc(dest: &mut File, toc: &List, indent: usize) -> Result<(), io::Error> {
    gen_list(&toc, indent, dest)
}

fn gen_content(dest: &mut File, content: &Vec<Block>, indent: usize) -> Result<(), io::Error> {
    for block in content {
        match block {
            Header { spans, level, id } => { gen_header(spans, level, id, indent, dest)?; },
            Blockquote { lines } => { gen_blockquote(lines, indent, dest)?; },
            ListElement(list) => { gen_list(list, indent, dest)?; },
            LinkCard { title, image, url, description, site_name } => { gen_link_card(title, image, url, description, site_name, indent, dest)?; },
            Table { head, body } => { gen_table(head, body, indent, dest)?; },
            Paragraph { spans } => { gen_paragraph(spans, indent, dest)?; },
            MathBlock { math } => { gen_math_block(math, indent, dest)?; },
            CodeBlock { lang, code } => { gen_code_block(lang, code, indent, dest)?; },
        }
    }
    Ok(())
}

fn gen_header(spans: &Vec<Span>, level: &u32, id: &String, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "{:>indent$}<h{} id=\"{}\">", " ", *level, *id)?;
    gen_spans(spans, dest)?;
    writeln!(dest, "</h{}>", *level)
}

fn gen_blockquote(lines: &Vec<Vec<Span>>, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    writeln!(dest, "{:>indent$}<blockquote>", " ")?;
    for spans in lines {
        write!(dest, "{:>indent$}  <p>", " ")?;
        gen_spans(spans, dest)?;
        writeln!(dest, "</p>")?;
    }
    writeln!(dest, "{:>indent$}</blockquote>", " ")
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

fn gen_link_card(title: &String, image: &Option<String>, url: &String, description: &Option<String>, site_name: &Option<String>, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    writeln!(dest, "{:>indent$}<div class=\"linkcard\"><a class=\"linkcard-link\" href=\"{}\">", "", url)?;
    writeln!(dest, "{:>indent$}  <div class=\"linkcard-text\">", "")?;
    writeln!(dest, "{:>indent$}    <h3 class=\"linkcard-title\">{}</h3>", "", title)?;
    if let Some(desc) = description {
        writeln!(dest, "{:>indent$}    <p class=\"linkcard-description\">{}</p>", "", desc)?;
    }
    writeln!(dest, "{:>indent$}    <img  class=\"linkcard-favicon\" src=\"http://www.google.com/s2/favicons?domain={}\"><span  class=\"linkcard-sitename\">{}</span>", "", url, site_name.clone().unwrap_or(url.clone()))?;
    writeln!(dest, "{:>indent$}  </div>", "")?;
    if let Some(img) = image {
        writeln!(dest, "{:>indent$}  <img class=\"linkcard-image\" src=\"{}\">", "", img)?;
    }
    writeln!(dest, "{:>indent$}</a></div>", "")
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

fn gen_math_block(math: &String, indent: usize, dest: &mut File) -> Result<(), io::Error> {
    writeln!(dest, "{:>indent$}<p>\\[{}\\]</p>", " ", math)
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
            Link { text, url } => { gen_link(text, url, dest)?; },
            Emphasis { text } => { gen_emphasis(text, dest)?; },
            Strong { text } => { gen_strong(text, dest)?; },
            Math { math } => { gen_math(math, dest)?; },
            Code { code } => { gen_code(code, dest)?; },
            Image { url } => { gen_image(url, dest)?; },
            Text { text } => { gen_text(text, dest)?; },
        }
    }
    Ok(())
}

fn gen_link(text: &String, url: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "<a href=\"{}\">{}</a>", *url, *text)
}

fn gen_emphasis(text: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "<em>{}</em>", *text)
}

fn gen_strong(text: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "<strong>{}</strong>", *text)
}

fn gen_math(math: &String, dest: &mut File) -> Result<(), io::Error> {
    write!(dest, "\\({}\\)", *math)
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