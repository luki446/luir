use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralType {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Local,

    Identifier(String),

    // Literals
    Literal(LiteralType),

    // Other
    Plus,
    Minus,
    Asterisk,
    Slash,
    LeftParen,
    RightParen,
    Assigment,
    Dot,
    Comma,

    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    Concatanation,

    If,
    Then,
    Else,
    ElseIf,
    End,

    While,
    For,
    Do,
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

    fn consume_identifier_or_keyword(&mut self) -> Token {
        let id = self.consume_while(|c| c.is_alphanumeric() || c == '_');
        match id.as_str() {
            "local" => Token::Local,
            "nil" => Token::Literal(LiteralType::Nil),
            "true" => Token::Literal(LiteralType::Boolean(true)),
            "false" => Token::Literal(LiteralType::Boolean(false)),
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "elseif" => Token::ElseIf,
            "end" => Token::End,
            "while" => Token::While,
            "for" => Token::For,
            "do" => Token::Do,
            _ => Token::Identifier(id),
        }
    }

    fn consume_number(&mut self) -> Result<Token, String> {
        let num_str = self.consume_while(|c| c.is_ascii_digit() || c == '.');
        Ok(Token::Literal(LiteralType::Number(
            num_str.parse().or(Err("Number conversion error"))?,
        )))
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
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
                '<' => {
                    if Some('=') == self.input.clone().next() {
                        tokens.push(Token::LessThanOrEqual);
                        self.advance();
                    } else {
                        tokens.push(Token::LessThan);
                    }

                    self.advance();
                }
                '>' => {
                    if Some('=') == self.input.clone().next() {
                        tokens.push(Token::GreaterThanOrEqual);
                        self.advance();
                    } else {
                        tokens.push(Token::GreaterThan);
                    }

                    self.advance();
                }
                '=' => {
                    if Some('=') == self.input.clone().next() {
                        tokens.push(Token::Equal);
                        self.advance();
                    } else {
                        tokens.push(Token::Assigment);
                    }

                    self.advance();
                }

                '~' => {
                    if Some('=') == self.input.clone().next() {
                        tokens.push(Token::NotEqual);
                        self.advance();
                    } else {
                        return Err(String::from("Unexpected char after ~ expected ="));
                    }

                    self.advance();
                }

                '.' => {
                    if Some('.') == self.input.clone().next() {
                        tokens.push(Token::Concatanation);
                        self.advance();
                    } else {
                        tokens.push(Token::Dot);
                    }

                    self.advance();
                }

                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }

                _ if c.is_whitespace() => {
                    self.consume_whitespace();
                }
                _ if c.is_ascii_digit() => {
                    tokens.push(self.consume_number()?);
                }
                _ if c.is_ascii_alphabetic() => {
                    tokens.push(self.consume_identifier_or_keyword());
                }
                '"' => {
                    self.advance();
                    let string = self.consume_while(|c| c != '"');
                    self.advance();
                    tokens.push(Token::Literal(LiteralType::String(string)));
                }
                _ => Err(format!("Unexpected character: {}", c))?,
            }
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod lex_tests {
    use super::*;

    #[test]
    fn test_basic_variable_declaration_lexer() {
        let source_code = "local a = 1 + 2 * 3";
        let mut lexer = Lexer::new(source_code);

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("a".to_string()),
                Token::Assigment,
                Token::Literal(LiteralType::Number(1.0)),
                Token::Plus,
                Token::Literal(LiteralType::Number(2.0)),
                Token::Asterisk,
                Token::Literal(LiteralType::Number(3.0)),
            ]
        );
    }

    #[test]
    fn test_string_literal_lexer() {
        let source_code = r#"print("Hello, World!")"#;
        let mut lexer = Lexer::new(source_code);

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("print".to_string()),
                Token::LeftParen,
                Token::Literal(LiteralType::String("Hello, World!".to_string())),
                Token::RightParen,
            ]
        );
    }

    #[test]
    fn test_whitespace_lexer() {
        let source_code = "a = 1   +   2";
        let mut lexer = Lexer::new(source_code);

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::Assigment,
                Token::Literal(LiteralType::Number(1.0)),
                Token::Plus,
                Token::Literal(LiteralType::Number(2.0)),
            ]
        );
    }

    #[test]
    fn test_comparison_lexer() {
        let source_code = "a >= 1";
        let mut lexer = Lexer::new(source_code);

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::GreaterThanOrEqual,
                Token::Literal(LiteralType::Number(1.0)),
            ]
        );
    }

    #[test]
    fn test_not_equal_lexer() {
        let source_code = "a ~= 1";
        let mut lexer = Lexer::new(source_code);

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::NotEqual,
                Token::Literal(LiteralType::Number(1.0)),
            ]
        );
    }

    #[test]
    fn test_concatenation_lexer() {
        let source_code = "a .. b";
        let mut lexer = Lexer::new(source_code);

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::Concatanation,
                Token::Identifier("b".to_string()),
            ]
        );
    }
}
