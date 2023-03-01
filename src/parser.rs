use crate::{
    ast,
    lex::{self, Lexer, LiteralType},
};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source_code),
        }
    }

    pub fn parse(&mut self) -> Result<ast::Block, String> {
        let mut statements = Vec::new();

        let tokens = self.lexer.tokenize()?;
        let mut tokens = tokens.into_iter().peekable();

        while let Some(token) = tokens.peek() {
            match token {
                lex::Token::Local => {
                    let local_variable_declaration =
                        self.parse_local_variable_declaration(&mut tokens)?;

                    statements.push(local_variable_declaration);
                }
                _ => return Err(format!("Unexpected token '{:?}'", token)),
            }
        }

        Ok(ast::Block::new(statements))
    }

    fn parse_local_variable_declaration(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Box<dyn ast::Statement>, String> {
        let mut local_variable_declaration = ast::LocalVariableDeclaration::new();

        tokens.next();

        if let Some(lex::Token::Identifier(identifier)) = tokens.next() {
            local_variable_declaration.set_identifier(identifier);
        } else {
            return Err("Expected identifier".to_string());
        }

        if let Some(lex::Token::Assigment) = tokens.next() {
            let expression = self.parse_expression(tokens)?;

            local_variable_declaration.set_expression(expression);
        } else {
            return Err("Expected '='".to_string());
        }

        Ok(Box::new(local_variable_declaration))
    }

    fn parse_expression(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Box<dyn ast::Expression>, String> {
        let mut left = self.parse_term(tokens)?;

        while let Some(token) = tokens.peek() {
            match token {
                lex::Token::Plus => {
                    tokens.next();

                    let right = self.parse_term(tokens)?;

                    left = Box::new(ast::BinaryExpression::from(left, right, "+".to_string()));
                }
                lex::Token::Minus => {
                    tokens.next();

                    let right = self.parse_term(tokens)?;

                    left = Box::new(ast::BinaryExpression::from(left, right, "-".to_string()));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_term(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Box<dyn ast::Expression>, String> {
        let mut left = self.parse_factor(tokens)?;

        while let Some(token) = tokens.peek() {
            match token {
                lex::Token::Asterisk => {
                    tokens.next();

                    let right = self.parse_factor(tokens)?;

                    left = Box::new(ast::BinaryExpression::from(left, right, "*".to_string()));
                }
                lex::Token::Slash => {
                    tokens.next();

                    let right = self.parse_factor(tokens)?;

                    left = Box::new(ast::BinaryExpression::from(left, right, "/".to_string()));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_factor(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Box<dyn ast::Expression>, String> {
        if let Some(token) = tokens.next() {
            match token {
                lex::Token::LeftParen => {
                    let expression = self.parse_expression(tokens)?;

                    if let Some(lex::Token::RightParen) = tokens.next() {
                        Ok(expression)
                    } else {
                        Err("Expected ')'".to_string())
                    }
                }
                lex::Token::Literal(LiteralType::Number(number)) => {
                    Ok(Box::new(ast::NumberExpression::new(number)))
                }
                lex::Token::Identifier(identifier) => {
                    Ok(Box::new(ast::IdentifierExpression::new(identifier)))
                }
                _ => Err(format!("Unexpected token '{:?}'", token)),
            }
        } else {
            Err("Expected factor".to_string())
        }
    }
}
