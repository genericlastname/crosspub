use std::io::{BufRead, BufReader};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenKind {
    Text,
    Link,
    UnorderedList,
    Blockquote,
    Heading,
    SubHeading,
    SubSubHeading,
    PreFormattedText,
}

#[derive(Clone)]
pub struct GemtextToken {
    pub kind: TokenKind,
    pub data: String,
    pub extra: String,  // Right now this will be empty except when links are
                        // named, when it will hold the user friendly name.
}

// Returns a Vec<&str> from a given str with newline and linefeed bytes
// maintained.
fn split_keep_crlf(raw_text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut buflen: usize;
    let mut current: String = String::new();
    let mut reader = BufReader::new(raw_text.as_bytes());
    loop {
        buflen = reader.read_line(&mut current)
            .expect("Couldn't read buffer.");
        if buflen == 0 { break; }
        let copy = current.clone();
        current.clear();
        lines.push(copy);
    }
    lines
}

// Take in a string of gemtext and convert it into a vector of GemtextTokens
// with a kind and data.
pub fn parse_gemtext(raw_text: &str) -> Vec<GemtextToken> {
    let mut gemtext_token_chain = Vec::new();
    let raw_text_lines: Vec<String> = split_keep_crlf(raw_text);
    let mut current_pft_state: bool = false;
    let mut pft_block = String::new();
    let mut _pft_alt_text: &str = "";

    for line in raw_text_lines {
        let mut mode: TokenKind;
        let text_tokens: Vec<&str> = line.splitn(3, ' ').collect();

        if !current_pft_state {
            match text_tokens[0] {
                "=>"  => { mode = TokenKind::Link; },
                "*"   => { mode = TokenKind::UnorderedList; },
                ">"   => { mode = TokenKind::Blockquote; },
                "###" => { mode = TokenKind::SubSubHeading; },
                "##"  => { mode = TokenKind::SubHeading; },
                "#"   => { mode = TokenKind::Heading; },
                _     => {
                    mode = TokenKind::Text;
                },
            }
            if text_tokens[0].starts_with("```") {
                mode = TokenKind::PreFormattedText;
            }

            match text_tokens.len() {
                3 => {
                    if mode == TokenKind::Link {
                        gemtext_token_chain.push(GemtextToken {
                            kind: mode,
                            data: text_tokens[1].to_owned(),
                            extra: text_tokens[2].to_owned(),
                        });
                    } else if mode == TokenKind::Text {
                        // Combine [0], [1], and [2] since Text doesn't have a
                        // leading symbol.
                        gemtext_token_chain.push(GemtextToken {
                            kind: mode,
                            data: format!("{} {} {}",
                                text_tokens[0],
                                text_tokens[1],
                                text_tokens[2]),
                                extra: "".to_owned(),
                        });
                    } else {
                        // Combine [1] and [2] in other parse modes.
                        gemtext_token_chain.push(GemtextToken {
                            kind: mode,
                            data: format!("{} {}",
                                text_tokens[1],
                                text_tokens[2]),
                                extra: "".to_owned(),
                        });
                    }
                },
                2 => {
                    if mode == TokenKind::PreFormattedText && !current_pft_state {
                        current_pft_state = true;
                        _pft_alt_text = text_tokens[1];
                    }
                    else {
                        gemtext_token_chain.push(GemtextToken {
                            kind: mode,
                            data: text_tokens[1].to_owned(),
                            extra: "".to_owned(),
                        });
                    }
                },
                _ => {
                    if mode == TokenKind::PreFormattedText && !current_pft_state {
                        current_pft_state = true;
                    } else {
                        gemtext_token_chain.push(GemtextToken {
                            kind: mode,
                            data: text_tokens[0].to_owned(),
                            extra: "".to_owned(),
                        });
                    }
                }
            }
        } else {
            if text_tokens[0].starts_with("```") {
                let pft_block_copy = pft_block.clone();
                pft_block.clear();
                current_pft_state = false;
                // TODO: Support PFT alt text.
                gemtext_token_chain.push(GemtextToken {
                    kind: TokenKind::PreFormattedText,
                    data: pft_block_copy,
                    extra: "".to_owned(),
                });
            } else {
                pft_block.push_str(&line);
            }
        }
    }

    gemtext_token_chain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_handles_text() {
        let text = "Hello world this is example text";
        let parsed: Vec<GemtextToken> = parse_gemtext(text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind, TokenKind::Text);
        assert_eq!(parsed[0].data, text);
    }

    #[test]
    fn parser_handles_links() {
        let raw_text = "=> www.example.com";
        let text_data = "www.example.com";
        let parsed: Vec<GemtextToken> = parse_gemtext(raw_text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind, TokenKind::Link);
        assert_eq!(parsed[0].data, text_data);
    }

    #[test]
    fn parser_handles_links_with_names() {
        let raw_text = "=> www.example.com Example Link";
        let text_data = "www.example.com";
        let extra_data = "Example Link";
        let parsed: Vec<GemtextToken> = parse_gemtext(raw_text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind, TokenKind::Link);
        assert_eq!(parsed[0].data, text_data);
        assert_eq!(parsed[0].extra, extra_data);
    }

    #[test]
    fn parser_handles_lists() {
        let raw_text = "* Item";
        let text_data = "Item";  // The text data after parsing.
        let parsed: Vec<GemtextToken> = parse_gemtext(raw_text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind, TokenKind::UnorderedList);
        assert_eq!(parsed[0].data, text_data);
    }

    #[test]
    fn parser_handles_blockquotes() {
        let raw_text = "> block quote";
        let text_data = "block quote";
        let parsed: Vec<GemtextToken> = parse_gemtext(raw_text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind, TokenKind::Blockquote);
        assert_eq!(parsed[0].data, text_data);
    }

    #[test]
    fn parser_handles_headings() {
        let raw_text =
            "\
            # Heading\n\
            ## SubHeading\n\
            ### SubSubHeading";
        let line0 = "Heading\n";
        let line1 = "SubHeading\n";
        let line2 = "SubSubHeading";
        let parsed: Vec<GemtextToken> = parse_gemtext(raw_text);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].kind, TokenKind::Heading);
        assert_eq!(parsed[1].kind, TokenKind::SubHeading);
        assert_eq!(parsed[2].kind, TokenKind::SubSubHeading);
        assert_eq!(parsed[0].data, line0);
        assert_eq!(parsed[1].data, line1);
        assert_eq!(parsed[2].data, line2);
    }

    #[test]
    fn parser_handles_pft() {
        let raw_text =
            "```\n\
            This text is unformatted.\n\
            This is the second line.\n\
            ```";
        let line = "This text is unformatted.\nThis is the second line.\n";
        let parsed: Vec<GemtextToken> = parse_gemtext(raw_text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind, TokenKind::PreFormattedText);
        assert_eq!(parsed[0].data, line);
    }
}
