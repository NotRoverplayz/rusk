
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    OpenTag(String),
    CloseTag(String),
    SelfClosingTag(String),
    Comment,

    AttributeName(String),
    AttributeValue(String),

    Text(String),

    Equals,
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current()?;
        self.position += 1;
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while self.current().is_some_and(|c| c.is_whitespace()) {
            self.advance();
        }
    }

    fn read_name(&mut self) -> String {
        let mut name = String::new();

        while let Some(ch) = self.current() {
            if ch.is_whitespace()
                || ch == '>'
                || ch == '='
                || ch == '/'
                || ch == '<'
            {
                break;
            }

            name.push(ch);
            self.advance();
        }

        name
    }

    fn read_value(&mut self) -> String {
        let mut value = String::new();

        if let Some(quote @ ('"' | '\'')) = self.current() {
            self.advance();

            while let Some(ch) = self.current() {
                self.advance();

                if ch == quote {
                    break;
                }

                value.push(ch);
            }
        } else {
            while let Some(ch) = self.current() {
                if ch.is_whitespace() || ch == '>' || ch == '/' {
                    break;
                }

                value.push(ch);
                self.advance();
            }
        }

        value
    }

    fn read_text(&mut self) -> String {
        let mut text = String::new();

        while let Some(ch) = self.current() {
            if ch == '<' {
                break;
            }

            text.push(ch);
            self.advance();
        }

        text
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.current() {
            if ch == '<' {
                if self.input[self.position..].starts_with(&['<', '!', '-', '-']) {
                    self.position += 4;
                    while self.position + 2 < self.input.len()
                        && !self.input[self.position..self.position + 3].starts_with(&['-', '-', '>'])
                    {
                        self.position += 1;
                    }
                    if self.position + 2 < self.input.len() {
                        self.position += 3;
                    }
                    tokens.push(Token::Comment);
                    continue;
                }

                self.advance();

                let is_closing = self.current() == Some('/');

                if is_closing {
                    self.advance();
                }

                let name = self.read_name();

                if name.is_empty() {
                    continue;
                }

                if is_closing {
                    self.skip_whitespace();

                    while let Some(ch) = self.current() {
                        self.advance();

                        if ch == '>' {
                            break;
                        }
                    }

                    tokens.push(Token::CloseTag(name));
                    continue;
                }

                // Collect attributes before deciding
                // whether this is a normal or self-closing tag.
                let mut attributes = Vec::new();
                let mut self_closing = false;

                loop {
                    self.skip_whitespace();

                    match self.current() {
                        Some('>') => {
                            self.advance();
                            break;
                        }

                        Some('/') => {
                            self.advance();

                            self.skip_whitespace();

                            if self.current() == Some('>') {
                                self.advance();
                                self_closing = true;
                                break;
                            }
                        }

                        Some(_) => {
                            let attribute_name = self.read_name();

                            if attribute_name.is_empty() {
                                self.advance();
                                continue;
                            }

                            attributes.push(
                                Token::AttributeName(attribute_name)
                            );

                            self.skip_whitespace();

                            if self.current() == Some('=') {
                                self.advance();
                                attributes.push(Token::Equals);

                                self.skip_whitespace();

                                let value = self.read_value();

                                attributes.push(
                                    Token::AttributeValue(value)
                                );
                            }
                        }

                        None => break,
                    }
                }

                if self_closing {
                    tokens.push(Token::SelfClosingTag(name));
                } else {
                    tokens.push(Token::OpenTag(name));
                }

                tokens.extend(attributes);
            } else {
                let text = self.read_text();

                if !text.trim().is_empty() {
                    tokens.push(Token::Text(text));
                }
            }
        }

        tokens.push(Token::Eof);
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsml_lexer() {
        let source = r#"
            <window title="RUSK Website">
                <text>Hello from RSML!</text>
                <button text="Click me"/>
            </window>
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        println!("{:#?}", tokens);

        assert!(tokens.contains(
            &Token::OpenTag("window".to_string())
        ));

        assert!(tokens.contains(
            &Token::AttributeName("title".to_string())
        ));

        assert!(tokens.contains(
            &Token::AttributeValue("RUSK Website".to_string())
        ));

        assert!(tokens.contains(
            &Token::SelfClosingTag("button".to_string())
        ));

        assert!(tokens.contains(
            &Token::CloseTag("window".to_string())
        ));
    }

    #[test]
    fn test_single_quotes() {
        let source = r#"<text color='purple'>Hello</text>"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert!(tokens.contains(
            &Token::AttributeValue("purple".to_string())
        ));
    }

    #[test]
    fn ignores_rsml_comments() {
        let mut lexer = Lexer::new(r#"<!-- navigation --><text>Hello</text>"#);
        let tokens = lexer.tokenize();

        assert!(tokens.contains(&Token::Comment));
        assert!(tokens.contains(&Token::Text("Hello".to_string())));
    }
}
