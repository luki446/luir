use crate::{
    ast,
    lex::{self, Lexer, LiteralType},
};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

macro_rules! create_binary_expression {
    ($parser:expr, $tokens:expr, $parse_next_level_expression:expr, [$( ($op:path, $op_str:expr) ),+]) => {{
        let mut left = $parse_next_level_expression($parser, $tokens)?;

        while let Some(token) = $tokens.peek() {
            match token {
                $( $op => {
                    $tokens.next();
                    let right = $parse_next_level_expression($parser, $tokens)?;
                    left = Box::new(ast::BinaryExpression::from(left, right, $op_str.to_string()));
                }, )+
                _ => break,
            }
        }

        Ok(left)
    }};
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
        create_binary_expression!(
            self,
            tokens,
            Self::parse_2_level_expression,
            [
                (lex::Token::NotEqual, "~="),
                (lex::Token::Equal, "=="),
                (lex::Token::LessThan, "<"),
                (lex::Token::LessThanOrEqual, "<="),
                (lex::Token::GreaterThan, ">"),
                (lex::Token::GreaterThanOrEqual, ">=")
            ]
        )
    }

    fn parse_2_level_expression(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Box<dyn ast::Expression>, String> {
        create_binary_expression!(
            self,
            tokens,
            Self::parse_3_level_expression,
            [(lex::Token::Plus, "+"), (lex::Token::Minus, "-")]
        )
    }

    fn parse_3_level_expression(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Box<dyn ast::Expression>, String> {
        create_binary_expression!(
            self,
            tokens,
            Self::parse_4_level_expression,
            [(lex::Token::Asterisk, "*"), (lex::Token::Slash, "/")]
        )
    }

    fn parse_4_level_expression(
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
                    Ok(Box::new(ast::NumberLiteral::new(number)))
                }
                lex::Token::Identifier(identifier) => {
                    Ok(Box::new(ast::IdentifierExpression::new(identifier)))
                }
                lex::Token::Literal(LiteralType::Boolean(value)) => {
                    Ok(Box::new(ast::BooleanLiteral::new(value)))
                }
                lex::Token::Literal(LiteralType::Nil) => Ok(Box::new(ast::NilLiteral {})),
                lex::Token::Literal(LiteralType::String(value)) => {
                    Ok(Box::new(ast::StringLiteral::new(value)))
                }
                _ => Err(format!("Unexpected token '{:?}'", token)),
            }
        } else {
            Err("Expected factor".to_string())
        }
    }
}
