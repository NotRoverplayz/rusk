
use crate::token::Token;

pub struct Lexer {
    source: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
        }
    }

    fn current(&self) -> char {
        self.source.get(self.position).copied().unwrap_or('\0')
    }

    fn advance(&mut self) -> char {
        let character = self.current();

        if character != '\0' {
            self.position += 1;
        }

        character
    }

    fn skip_spaces(&mut self) {
        while matches!(self.current(), ' ' | '\t' | '\r') {
            self.advance();
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;

        while self.current().is_ascii_digit() || self.current() == '.' {
            self.advance();
        }

        let value: String = self.source[start..self.position].iter().collect();

        Token::Number(value.parse().unwrap_or(0.0))
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;

        while self.current().is_alphanumeric() || self.current() == '_' {
            self.advance();
        }

        let value: String = self.source[start..self.position].iter().collect();

        match value.as_str() {
            "let" => Token::Let,
            "set" => Token::Set,
            "to" => Token::To,
            "be" => Token::Be,
            "say" => Token::Say,
            "function" => Token::Function,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "end" => Token::End,
            "repeat" => Token::Repeat,
            "times" => Token::Times,
            "true" => Token::True,
            "false" => Token::False,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            _ => Token::Identifier(value),
        }
    }

    fn read_string(&mut self) -> Result<Token, String> {
        self.advance();

        let mut value = String::new();

        while self.current() != '"' && self.current() != '\0' {
            let character = self.advance();

            if character == '\\' {
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    other => value.push(other),
                }
            } else {
                value.push(character);
            }
        }

        if self.current() != '"' {
            return Err("Unterminated string".to_string());
        }

        self.advance();

        Ok(Token::String(value))
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.current() != '\0' {
            self.skip_spaces();

            match self.current() {
                '\0' => break,

                '\n' => {
                    self.advance();
                    tokens.push(Token::Newline);
                }

                '#' => {
                    while self.current() != '\n' && self.current() != '\0' {
                        self.advance();
                    }
                }

                '"' => tokens.push(self.read_string()?),

                c if c.is_ascii_digit() => tokens.push(self.read_number()),

                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.read_identifier());
                }

                '(' => {
                    self.advance();
                    tokens.push(Token::LeftParen);
                }

                ')' => {
                    self.advance();
                    tokens.push(Token::RightParen);
                }

                '{' => {
                    self.advance();
                    tokens.push(Token::LeftBrace);
                }

                '}' => {
                    self.advance();
                    tokens.push(Token::RightBrace);
                }

                '[' => {
                    self.advance();
                    tokens.push(Token::LeftBracket);
                }

                ']' => {
                    self.advance();
                    tokens.push(Token::RightBracket);
                }

                ',' => {
                    self.advance();
                    tokens.push(Token::Comma);
                }

                '+' => {
                    self.advance();
                    tokens.push(Token::Plus);
                }

                '-' => {
                    self.advance();
                    tokens.push(Token::Minus);
                }

                '*' => {
                    self.advance();
                    tokens.push(Token::Star);
                }

                '/' => {
                    self.advance();
                    tokens.push(Token::Slash);
                }

                '=' => {
                    self.advance();

                    if self.current() == '=' {
                        self.advance();
                        tokens.push(Token::EqualEqual);
                    } else {
                        tokens.push(Token::Equal);
                    }
                }

                '!' => {
                    self.advance();

                    if self.current() == '=' {
                        self.advance();
                        tokens.push(Token::NotEqual);
                    } else {
                        return Err("Expected '=' after '!'".to_string());
                    }
                }

                '<' => {
                    self.advance();

                    if self.current() == '=' {
                        self.advance();
                        tokens.push(Token::LessEqual);
                    } else {
                        tokens.push(Token::Less);
                    }
                }

                '>' => {
                    self.advance();

                    if self.current() == '=' {
                        self.advance();
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::Greater);
                    }
                }

                other => {
                    return Err(format!("Unexpected character '{}'", other));
                }
            }
        }

        tokens.push(Token::Eof);

        Ok(tokens)
    }
}
