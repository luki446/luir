use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum Token {
    // Keywords
    Local,

    Identifier(String),

    // Literals
    NumberLiteral(f64),

    // Other
    Plus,
    Minus,
    Asterisk,
    Slash,
    LeftParen,
    RightParen,
    Equal,
}

pub struct Lexer<'a> {
    input: Chars<'a>,
    current: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer {
        let mut chars = input.chars();
        Lexer {
            current: chars.next(),
            input: chars,
        }
    }

    fn advance(&mut self) {
        self.current = self.input.next();
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(c) = self.current {
            if test(c) {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    fn consume_identifier(&mut self) -> Token {
        let id = self.consume_while(|c| c.is_alphanumeric() || c == '_');
        match id.as_str() {
            "local" => Token::Local,
            _ => Token::Identifier(id),
        }
    }

    fn consume_number(&mut self) -> Token {
        let num_str = self.consume_while(|c| c.is_ascii_digit() || c == '.');
        Token::NumberLiteral(num_str.parse().unwrap())
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        self.consume_whitespace();
        while let Some(c) = self.current {
            match c {
                '+' => {
                    tokens.push(Token::Plus);
                    self.advance();
                }
                '-' => {
                    tokens.push(Token::Minus);
                    self.advance();
                }
                '*' => {
                    tokens.push(Token::Asterisk);
                    self.advance();
                }
                '/' => {
                    tokens.push(Token::Slash);
                    self.advance();
                }
                '(' => {
                    tokens.push(Token::LeftParen);
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.advance();
                }
                '=' => {
                    tokens.push(Token::Equal);
                    self.advance();
                }
                _ if c.is_whitespace() => {
                    self.consume_whitespace();
                }
                _ if c.is_ascii_digit() => {
                    tokens.push(self.consume_number());
                }
                _ if c.is_ascii_alphabetic() => {
                    tokens.push(self.consume_identifier());
                }
                _ => panic!("Invalid character: {}", c),
            }
        }
        tokens
    }
}

#[cfg(test)]
mod lex_tests {
    use super::*;

    #[test]
    fn test_basic_variable_declaration_lexer() {
        let source_code = "local a = 1 + 2 * 3";
        let mut lexer = Lexer::new(source_code);

        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("a".to_string()),
                Token::Equal,
                Token::NumberLiteral(1.0),
                Token::Plus,
                Token::NumberLiteral(2.0),
                Token::Asterisk,
                Token::NumberLiteral(3.0),
            ]
        );
    }
}
