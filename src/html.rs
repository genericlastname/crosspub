use std::fmt::Write;
use crate::gemtext::{GemtextToken, TokenKind};

pub fn generate_html_from_tokens(tokens: Vec<GemtextToken>) -> String {
    let mut in_list = false;
    let mut buf: String;

    for token in tokens {
        if token.kind != TokenKind::UnorderedList && in_list {
            writeln!(buf, "</ul>");
            in_list = false;
        }

        match token.kind {
            TokenKind::Text => {
                writeln!(buf, "<p>{}</p>", token.data);
            }
            TokenKind::Link => {
                writeln!(buf, "<p><a href=\"{}\">{}</a></p>", token.data, token.extra);
            }
            TokenKind::UnorderedList => {
                if !in_list {
                    writeln!(buf, "<ul>");
                    in_list = true;
                }
                writeln!(buf, "<li>{}</li>", token.data);
            }
            TokenKind::Blockquote => {
                writeln!(buf, "<blockquote>{}</blockquote>", token.data);
            }
            TokenKind::Heading => {
                writeln!(buf, "<h1>{}</h1>", token.data);
            }
            TokenKind::SubHeading => {
                writeln!(buf, "<h2>{}</h2>", token.data);
            }
            TokenKind::SubSubHeading => {
                writeln!(buf, "<h3>{}</h3>", token.data);
            }
            TokenKind::PreFormattedText => {
                writeln!(buf, "<pre>{}</pre>", token.data);
            }
        }
    }
    buf
}
