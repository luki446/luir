use crate::{
    ast::{self, IfStatement},
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

    pub fn parse_file_level(&mut self) -> Result<ast::Block, String> {
        let mut statements = Vec::new();

        let tokens = self.lexer.tokenize()?;
        let mut tokens = tokens.into_iter().peekable();

        while let Some(_) = tokens.peek() {
            statements.push(self.parse_single_statement(&mut tokens)?);
        }

        Ok(ast::Block::new(statements))
    }

    fn parse_single_statement(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Box<dyn ast::Statement>, String> {
        let token = tokens.peek();

        match token {
            Some(lex::Token::Local) => self.parse_local_variable_declaration(tokens),
            Some(lex::Token::Identifier(_)) => {
                let expression = self.parse_expression(tokens)?;

                Ok(Box::new(ast::ExpressionStatement::new(expression)))
            }
            Some(lex::Token::If) => {
                tokens.next();

                let condition = self.parse_expression(tokens)?;

                self.expect(tokens, lex::Token::Then)?;

                let main_block = self.parse_block_until(tokens, &[lex::Token::End, lex::Token::ElseIf, lex::Token::Else])?;
                
                self.expect(tokens, lex::Token::End)?;

                Ok(Box::new(IfStatement::new(condition, main_block)))
            }
            _ => Err(format!("Unexpected token '{:?}'", token)),
        }
    }

    fn parse_block_until(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
        end_tokens: &[lex::Token],
    ) -> Result<ast::Block, String> {
        let mut statements = Vec::new();

        while let Some(token) = tokens.peek() {
            if end_tokens.contains(token) {
                break;
            }

            statements.push(self.parse_single_statement(tokens)?);
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
                    if tokens.peek() == Some(&lex::Token::LeftParen) {
                        tokens.next();

                        let mut arguments = Vec::new();

                        while let Ok(expression) = self.parse_expression(tokens) {
                            arguments.push(expression);

                            if let Some(lex::Token::Comma) = tokens.peek() {
                                tokens.next();
                            } else {
                                break;
                            }
                        }

                        if tokens.peek() != Some(&lex::Token::RightParen) {
                            return Err("Expected ')'".to_string());
                        }

                        tokens.next();

                        Ok(Box::new(ast::FunctionCall::new(identifier, arguments)))
                    } else {
                        Ok(Box::new(ast::IdentifierExpression::new(identifier)))
                    }
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

    fn expect(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
        expected: lex::Token
    ) -> Result<(), String> {
        let next_token = tokens.next();
        if  next_token == Some(expected.clone()) {
            Ok(())
        } else {
            Err(format!("Expected '{:?}'", expected))
        }
    }
}
