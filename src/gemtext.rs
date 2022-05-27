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

impl GemtextToken {
    pub fn as_html(&self) -> String {
        match self.kind {
            TokenKind::Heading => {
                format!("<h1>{}</h1>\n", self.data)
            },
            TokenKind::SubHeading => {
                format!("<h2>{}</h2>\n", self.data)
            },
            TokenKind::SubSubHeading => {
                format!("<h3>{}</h3>\n", self.data)
            },
            TokenKind::Link => {
                if self.extra.is_empty() {
                    format!("<p><a href=\"{}\">{}</a></p>\n", self.data, self.data)
                } else {
                    format!("<p><a href=\"{}\">{}</a></p>\n", self.data, self.extra)
                }
            },
            TokenKind::Blockquote => {
                format!("<blockquote>{}</blockquote>\n", self.data)
            },
            TokenKind::PreFormattedText => {
                format!("<pre>{}</pre>\n", self.data)
            },
            TokenKind::UnorderedList => {
                format!("<li>{}</li>\n", self.data)
            }
            TokenKind::Text => {
                if !self.data.is_empty() {
                    format!("<p>{}</p>\n", self.data)
                } else {
                    String::new()
                }
            }
        }
    }
}

// Take in a string of gemtext and convert it into a vector of GemtextTokens
// with a kind and data.
pub fn parse_gemtext(lines: &[String]) -> Vec<GemtextToken> {
    let mut gemtext_token_chain = Vec::new();
    let mut current_pft_state: bool = false;
    let mut pft_block = String::new();
    let mut _pft_alt_text: &str = "";

    for line in lines {
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
