
use super::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
}

impl Element {
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element(Element),
    Text(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        self.position += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let actual = self.advance();

        if actual == expected {
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?}",
                expected, actual
            ))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();

        while !matches!(self.current(), Token::Eof) {
            if let Some(node) = self.parse_node()? {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    fn parse_node(&mut self) -> Result<Option<Node>, String> {
        match self.current().clone() {
            Token::OpenTag(name) => {
                self.advance();
                let attributes = self.parse_attributes()?;

                let mut element = Element {
                    name,
                    attributes,
                    children: Vec::new(),
                };

                while !matches!(
                    self.current(),
                    Token::CloseTag(_) | Token::Eof
                ) {
                    if let Some(child) = self.parse_node()? {
                        element.children.push(child);
                    }
                }

                match self.advance() {
                    Token::CloseTag(close_name) => {
                        if close_name != element.name {
                            return Err(format!(
                                "Mismatched closing tag: expected </{}>, found </{}>",
                                element.name,
                                close_name
                            ));
                        }
                    }

                    other => {
                        return Err(format!(
                            "Expected closing tag for {}, found {:?}",
                            element.name,
                            other
                        ));
                    }
                }

                Ok(Some(Node::Element(element)))
            }

            Token::SelfClosingTag(name) => {
                self.advance();
                let attributes = self.parse_attributes()?;

                Ok(Some(Node::Element(Element {
                    name,
                    attributes,
                    children: Vec::new(),
                })))
            }

            Token::Text(text) => {
                self.advance();
                Ok(Some(Node::Text(text)))
            }

            Token::Comment => {
                self.advance();
                self.parse_node()
            }

            Token::CloseTag(name) => {
                Err(format!("Unexpected closing tag: </{}>", name))
            }

            Token::Eof => Ok(None),

            other => Err(format!("Unexpected token: {:?}", other)),
        }
    }

    fn parse_attributes(&mut self) -> Result<Vec<(String, String)>, String> {
        let mut attributes = Vec::new();

        while let Token::AttributeName(_) = self.current() {
            let name = match self.advance() {
                Token::AttributeName(name) => name,
                _ => unreachable!(),
            };

            self.expect(Token::Equals)?;

            let value = match self.advance() {
                Token::AttributeValue(value) => value,
                other => {
                    return Err(format!(
                        "Expected attribute value, found {:?}",
                        other
                    ));
                }
            };

            attributes.push((name, value));
        }

        Ok(attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rsml::lexer::Lexer;

    #[test]
    fn parses_rsml_document() {
        let source = r#"
            <window title="RUSK Website">
                <text>Hello from RSML!</text>
                <button text="Click me"/>
            </window>
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        println!("{:#?}", result);

        assert!(result.is_ok());

        let nodes = result.unwrap();

        assert_eq!(nodes.len(), 1);
    }
}
