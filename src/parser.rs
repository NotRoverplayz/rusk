
use crate::ast::{Expr, Program, Statement};
use crate::token::Token;

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

        if token != Token::Eof {
            self.position += 1;
        }

        token
    }

    fn check(&self, expected: &Token) -> bool {
        self.current() == expected
    }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, got {:?}",
                expected,
                self.current()
            ))
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) {
            self.advance();
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let statements = self.parse_block(&[Token::Eof])?;
        Ok(Program { statements })
    }

    fn parse_block(&mut self, stops: &[Token]) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !stops.contains(self.current()) && !self.check(&Token::Eof) {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current() {
            Token::Let => self.parse_let(),
            Token::Set => self.parse_set(),
            Token::Say => self.parse_say(),
            Token::Function => self.parse_function(),
            Token::If => self.parse_if(),
            Token::Repeat => self.parse_repeat(),

            Token::Identifier(name) if name == "append" => {
                self.parse_append()
            }

            _ => {
                let expression = self.parse_expression()?;
                Ok(Statement::Expression(expression))
            }
        }
    }

    fn parse_let(&mut self) -> Result<Statement, String> {
        self.advance();

        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => return Err(format!("Expected variable name, got {:?}", other)),
        };

        self.consume(Token::Be)?;

        let value = self.parse_expression()?;

        Ok(Statement::Let(name, value))
    }

    fn parse_set(&mut self) -> Result<Statement, String> {
        self.advance();

        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => return Err(format!("Expected variable name after set, got {:?}", other)),
        };

        if self.check(&Token::LeftBracket) {
            self.advance();

            let index = self.parse_expression()?;

            self.consume(Token::RightBracket)?;
            self.consume(Token::To)?;

            let value = self.parse_expression()?;

            return Ok(Statement::SetIndex {
                name,
                index,
                value,
            });
        }

        self.consume(Token::To)?;

        let value = self.parse_expression()?;

        Ok(Statement::Set(name, value))
    }

    fn parse_append(&mut self) -> Result<Statement, String> {
        self.advance();

        let value = self.parse_expression()?;

        self.consume(Token::To)?;

        let array = match self.advance() {
            Token::Identifier(name) => name,
            other => return Err(format!("Expected array name after 'to', got {:?}", other)),
        };

        Ok(Statement::Append { value, array })
    }

    fn parse_say(&mut self) -> Result<Statement, String> {
        self.advance();

        let value = self.parse_expression()?;

        Ok(Statement::Say(value))
    }

    fn parse_function(&mut self) -> Result<Statement, String> {
        self.advance();

        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => return Err(format!("Expected function name, got {:?}", other)),
        };

        self.consume(Token::LeftParen)?;
        self.skip_newlines();

        let mut parameters = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                match self.advance() {
                    Token::Identifier(name) => parameters.push(name),
                    other => return Err(format!("Expected parameter, got {:?}", other)),
                }

                self.skip_newlines();

                if self.check(&Token::Comma) {
                    self.advance();
                    self.skip_newlines();
                } else {
                    break;
                }
            }
        }

        self.consume(Token::RightParen)?;
        self.consume(Token::LeftBrace)?;

        self.skip_newlines();

        let body = self.parse_block(&[Token::RightBrace])?;

        self.consume(Token::RightBrace)?;

        Ok(Statement::Function {
            name,
            parameters,
            body,
        })
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.advance();

        let condition = self.parse_expression()?;

        self.consume(Token::Then)?;
        self.skip_newlines();

        let then_branch = self.parse_block(&[Token::Else, Token::End])?;

        let else_branch = if self.check(&Token::Else) {
            self.advance();
            self.skip_newlines();
            self.parse_block(&[Token::End])?
        } else {
            Vec::new()
        };

        self.consume(Token::End)?;

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_repeat(&mut self) -> Result<Statement, String> {
        self.advance();

        let count = self.parse_expression()?;

        self.consume(Token::Times)?;
        self.skip_newlines();

        let body = self.parse_block(&[Token::End])?;

        self.consume(Token::End)?;

        Ok(Statement::Repeat { count, body })
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut expression = self.parse_and()?;

        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;

            expression = Expr::Binary {
                left: Box::new(expression),
                operator: "or".to_string(),
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut expression = self.parse_equality()?;

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;

            expression = Expr::Binary {
                left: Box::new(expression),
                operator: "and".to_string(),
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut expression = self.parse_comparison()?;

        loop {
            let operator = match self.current() {
                Token::EqualEqual => "==",
                Token::NotEqual => "!=",
                _ => break,
            };

            self.advance();
            let right = self.parse_comparison()?;

            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.to_string(),
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut expression = self.parse_term()?;

        loop {
            let operator = match self.current() {
                Token::Less => "<",
                Token::LessEqual => "<=",
                Token::Greater => ">",
                Token::GreaterEqual => ">=",
                _ => break,
            };

            self.advance();
            let right = self.parse_term()?;

            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.to_string(),
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expression = self.parse_factor()?;

        loop {
            let operator = match self.current() {
                Token::Plus => "+",
                Token::Minus => "-",
                _ => break,
            };

            self.advance();
            let right = self.parse_factor()?;

            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.to_string(),
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expression = self.parse_unary()?;

        loop {
            let operator = match self.current() {
                Token::Star => "*",
                Token::Slash => "/",
                _ => break,
            };

            self.advance();
            let right = self.parse_unary()?;

            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.to_string(),
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.check(&Token::Minus) || self.check(&Token::Not) {
            let operator = match self.advance() {
                Token::Minus => "-",
                Token::Not => "not",
                _ => unreachable!(),
            };

            let expression = self.parse_unary()?;

            return Ok(Expr::Unary {
                operator: operator.to_string(),
                expression: Box::new(expression),
            });
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr, String> {
        let mut expression = self.parse_primary()?;

        if let Expr::Variable(name) = &expression {
            if self.check(&Token::LeftParen) {
                let name = name.clone();

                self.advance();
                self.skip_newlines();

                let mut arguments = Vec::new();

                if !self.check(&Token::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        self.skip_newlines();

                        if self.check(&Token::Comma) {
                            self.advance();
                            self.skip_newlines();
                        } else {
                            break;
                        }
                    }
                }

                self.consume(Token::RightParen)?;

                expression = Expr::Call { name, arguments };
            }
        }

        while self.check(&Token::LeftBracket) {
            self.advance();
            self.skip_newlines();

            let index = self.parse_expression()?;

            self.skip_newlines();
            self.consume(Token::RightBracket)?;

            expression = Expr::Index {
                array: Box::new(expression),
                index: Box::new(index),
            };
        }

        Ok(expression)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Token::Number(value) => Ok(Expr::Number(value)),
            Token::String(value) => Ok(Expr::String(value)),
            Token::True => Ok(Expr::Boolean(true)),
            Token::False => Ok(Expr::Boolean(false)),
            Token::Identifier(name) => Ok(Expr::Variable(name)),

            Token::LeftBracket => {
                self.skip_newlines();

                let mut elements = Vec::new();

                if !self.check(&Token::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        self.skip_newlines();

                        if self.check(&Token::Comma) {
                            self.advance();
                            self.skip_newlines();
                        } else {
                            break;
                        }
                    }
                }

                self.consume(Token::RightBracket)?;

                Ok(Expr::Array(elements))
            }

            Token::LeftParen => {
                self.skip_newlines();

                let expression = self.parse_expression()?;

                self.skip_newlines();
                self.consume(Token::RightParen)?;

                Ok(expression)
            }

            other => Err(format!(
                "Unexpected expression token: {:?}",
                other
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn bundled_scripts_parse() {
        let scripts = [
            ("arrays", include_str!("../examples/arrays.rusk")),
            ("graphics", include_str!("../examples/graphics.rusk")),
            ("graphics demo", include_str!("../examples/graphics_demo.rusk")),
            ("hello", include_str!("../examples/hello.rusk")),
            ("set test", include_str!("../examples/set_test.rusk")),
            ("snake example", include_str!("../examples/snake.rusk")),
            ("test game", include_str!("../examples/testgame.rusk")),
            ("times table", include_str!("../examples/times-table.rusk")),
            ("snake app", include_str!("../devapps/snake.rusk")),
        ];

        for (name, source) in scripts {
            let tokens = Lexer::new(source)
                .tokenize()
                .unwrap_or_else(|error| panic!("{} failed to tokenize: {}", name, error));

            Parser::new(tokens)
                .parse()
                .unwrap_or_else(|error| panic!("{} failed to parse: {}", name, error));
        }
    }
}
