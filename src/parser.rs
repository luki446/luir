use std::collections::BTreeMap;

use crate::{
    ast::{Expression, Statement},
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
                    left = Expression::BinaryExpression(Box::new(left), $op_str.to_string(), Box::new(right));
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

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();

        let tokens = self.lexer.tokenize()?;
        let mut tokens = tokens.into_iter().peekable();

        while tokens.peek().is_some() {
            statements.push(self.parse_single_statement(&mut tokens)?);
        }

        Ok(statements)
    }

    fn parse_single_statement(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Statement, String> {
        let token = tokens.peek();

        match token {
            Some(lex::Token::Local) => self.parse_local_variable_declaration(tokens),
            Some(lex::Token::Identifier(_)) => {
                let mut future = tokens.clone();
                future.next();
                if future.peek() == Some(&lex::Token::Assigment) {
                    Ok(self.parse_assigment_statement(tokens)?)
                } else {
                    let expression = self.parse_expression(tokens)?;
                    Ok(Statement::ExpressionStatement(Box::new(expression)))
                }
            }
            Some(lex::Token::If) => self.parse_if_statement(tokens),
            Some(lex::Token::While) => self.parse_while_loop(tokens),
            Some(lex::Token::For) => self.parse_for_loop(tokens),
            Some(lex::Token::Function) => self.parse_function_declaration(tokens),
            Some(lex::Token::Return) => self.parse_return_statement(tokens),
            Some(lex::Token::Repeat) => self.parse_repeat_statement(tokens),
            _ => Err(format!("Unexpected top-level token '{:?}'", token)),
        }
    }

    fn parse_while_loop(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Statement, String> {
        tokens.next();

        let loop_condition = self.parse_expression(tokens)?;

        self.expect(tokens, lex::Token::Do)?;

        let loop_block = self.parse_block_until(tokens, &[lex::Token::End])?;

        self.expect(tokens, lex::Token::End)?;

        Ok(Statement::WhileLoop {
            loop_condition: Box::new(loop_condition),
            code_block: loop_block,
        })
    }

    fn parse_for_loop(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Statement, String> {
        tokens.next();

        let loop_variable = self.parse_identifier(tokens)?;

        self.expect(tokens, lex::Token::Assigment)?;

        let start_value = self.parse_expression(tokens)?;
        self.expect(tokens, lex::Token::Comma)?;

        let end_value = self.parse_expression(tokens)?;

        let mut step_value: Expression = Expression::NumberLiteral(1.0);

        if tokens.peek() == Some(&lex::Token::Comma) {
            tokens.next();
            step_value = self.parse_expression(tokens)?;
        }

        self.expect(tokens, lex::Token::Do)?;

        let loop_block = self.parse_block_until(tokens, &[lex::Token::End])?;

        self.expect(tokens, lex::Token::End)?;

        Ok(Statement::ForLoop {
            iterator_identifier: loop_variable,
            starting_value: Box::new(start_value),
            ending_value: Box::new(end_value),
            step_value: Box::new(step_value),
            code_block: loop_block,
        })
    }

    fn parse_if_statement(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Statement, String> {
        tokens.next();

        let condition = self.parse_expression(tokens)?;

        self.expect(tokens, lex::Token::Then)?;

        let main_block = self.parse_block_until(
            tokens,
            &[lex::Token::End, lex::Token::ElseIf, lex::Token::Else],
        )?;

        let mut elseif_statements = Vec::new();

        while let Some(lex::Token::ElseIf) = tokens.peek() {
            tokens.next();

            let condition = Box::new(self.parse_expression(tokens)?);

            self.expect(tokens, lex::Token::Then)?;

            let block = self.parse_block_until(
                tokens,
                &[lex::Token::End, lex::Token::ElseIf, lex::Token::Else],
            )?;

            elseif_statements.push((condition, block));
        }

        let else_block = if let Some(lex::Token::Else) = tokens.peek() {
            tokens.next();

            Some(self.parse_block_until(tokens, &[lex::Token::End])?)
        } else {
            None
        };

        self.expect(tokens, lex::Token::End)?;

        Ok(Statement::IfStatement {
            basic_condition: Box::new(condition),
            code_block: main_block,
            elseif_statements,
            else_block,
        })
    }

    fn parse_block_until(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
        end_tokens: &[lex::Token],
    ) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();

        while let Some(token) = tokens.peek() {
            if end_tokens.contains(token) {
                // tokens.next();
                break;
            }

            statements.push(self.parse_single_statement(tokens)?);
        }

        Ok(statements)
    }

    fn parse_local_variable_declaration(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Statement, String> {
        tokens.next();

        let local_variable_identifier = self.parse_identifier(tokens)?;

        self.expect(tokens, lex::Token::Assigment)?;

        let expression = self.parse_expression(tokens)?;

        Ok(Statement::LocalVariableDeclaration(
            local_variable_identifier,
            Box::new(expression),
        ))
    }

    fn parse_identifier(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<String, String> {
        if let Some(lex::Token::Identifier(identifier)) = tokens.next() {
            Ok(identifier)
        } else {
            Err("Expected identifier".to_string())
        }
    }

    fn parse_expression(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Expression, String> {
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
    ) -> Result<Expression, String> {
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
    ) -> Result<Expression, String> {
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
    ) -> Result<Expression, String> {
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
                lex::Token::LeftBracket => {
                    let table_literal = self.parse_table(tokens)?;

                    if let Some(lex::Token::RightBracket) = tokens.next() {
                        Ok(table_literal)
                    } else {
                        Err("Expected '}'".to_string())
                    }
                }
                lex::Token::Literal(LiteralType::Number(number)) => {
                    Ok(Expression::NumberLiteral(number))
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

                        Ok(Expression::FunctionCall(identifier, arguments))
                    } else {
                        Ok(Expression::IdentifierExpression(identifier))
                    }
                }
                lex::Token::Literal(LiteralType::Boolean(value)) => {
                    Ok(Expression::BooleanLiteral(value))
                }
                lex::Token::Literal(LiteralType::Nil) => Ok(Expression::NilLiteral {}),
                lex::Token::Literal(LiteralType::String(value)) => {
                    Ok(Expression::StringLiteral(value))
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
        expected: lex::Token,
    ) -> Result<(), String> {
        let next_token = tokens.next();
        if next_token == Some(expected.clone()) {
            Ok(())
        } else {
            Err(format!("Expected '{:?}'", expected))
        }
    }

    fn parse_assigment_statement(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Statement, String> {
        let identifier = self.parse_identifier(tokens)?;

        self.expect(tokens, lex::Token::Assigment)?;

        let expression = self.parse_expression(tokens)?;

        Ok(Statement::AssigmentStatement(
            identifier,
            Box::new(expression),
        ))
    }

    fn parse_function_declaration(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Statement, String> {
        tokens.next();

        let function_name = self.parse_identifier(tokens)?;

        self.expect(tokens, lex::Token::LeftParen)?;

        let mut function_arguments = Vec::new();

        while let Ok(identifier) = self.parse_identifier(tokens) {
            function_arguments.push(identifier);

            if let Some(lex::Token::Comma) = tokens.peek() {
                tokens.next();
            } else {
                break;
            }
        }

        self.expect(tokens, lex::Token::RightParen)?;

        let function_body = self.parse_block_until(tokens, &[lex::Token::End])?;

        self.expect(tokens, lex::Token::End)?;

        Ok(Statement::FunctionDeclaration {
            function_name,
            function_arguments,
            function_body,
        })
    }

    fn parse_return_statement(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Statement, String> {
        tokens.next();

        let expression = self.parse_expression(tokens)?;

        Ok(Statement::ReturnStatement(Box::new(expression)))
    }

    fn parse_repeat_statement(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Statement, String> {
        tokens.next();

        let code_block = self.parse_block_until(tokens, &[lex::Token::Until])?;

        self.expect(tokens, lex::Token::Until)?;

        let loop_condition = Box::new(self.parse_expression(tokens)?);

        Ok(Statement::RepeatUntilLoop {
            code_block,
            loop_condition,
        })
    }

    fn parse_table(
        &mut self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<lex::Token>>,
    ) -> Result<Expression, String> {
        tokens.next();

        let mut table_structure = BTreeMap::new();
        let mut in_table_index = 1;

        while let Some(token) = tokens.peek() {
            match *token {
                lex::Token::RightBracket => {
                    break;
                }
                lex::Token::Comma => {
                    tokens.next();
                } // Skip comma
                _ => {
                    let element = self.parse_expression(tokens)?;
                    table_structure
                        .insert(Expression::NumberLiteral(in_table_index as f64), element);
                    in_table_index += 1;
                }
            }
        }

        Ok(Expression::TableLiteral(table_structure))
    }
}
