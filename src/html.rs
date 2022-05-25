use std::fmt::Write;
use crate::gemtext::{GemtextToken, TokenKind};

use crate::settings::{Config, FMConfig};

pub fn create_html_post_header(fm: &FMConfig, config: &Config) -> String {
    // TODO: add link rel stylesheet, noting that path will need to be truncated.
    let header = format!(r#"<head>
    <base href="{}">
    <link rel="stylesheet" type="text/css" href="{}">
    <title>{} | {}</title>
    </head>"#,
    config.site_url,
    config.css,
    fm.name,
    config.site_name);

    header
}

pub fn generate_html_from_tokens(tokens: Vec<GemtextToken>) -> String {
    let mut in_list = false;
    let mut buf = String::new();

    for token in tokens {
        if token.kind != TokenKind::UnorderedList && in_list {
            writeln!(buf, "</ul>").unwrap();
            in_list = false;
        }

        match token.kind {
            TokenKind::Text => {
                writeln!(buf, "<p>{}</p>", token.data).unwrap();
            }
            TokenKind::Link => {
                writeln!(buf, "<p><a href=\"{}\">{}</a></p>", token.data, token.extra).unwrap();
            }
            TokenKind::UnorderedList => {
                if !in_list {
                    writeln!(buf, "<ul>").unwrap();
                    in_list = true;
                }
                writeln!(buf, "<li>{}</li>", token.data).unwrap();
            }
            TokenKind::Blockquote => {
                writeln!(buf, "<blockquote>{}</blockquote>", token.data).unwrap();
            }
            TokenKind::Heading => {
                writeln!(buf, "<h1>{}</h1>", token.data).unwrap();
            }
            TokenKind::SubHeading => {
                writeln!(buf, "<h2>{}</h2>", token.data).unwrap();
            }
            TokenKind::SubSubHeading => {
                writeln!(buf, "<h3>{}</h3>", token.data).unwrap();
            }
            TokenKind::PreFormattedText => {
                writeln!(buf, "<pre>{}</pre>", token.data).unwrap();
            }
        }
    }
    buf
}
