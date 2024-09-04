use std::iter::Peekable;
use std::str::Chars;

use crate::token::{Token, TokenType};

#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    pub line: usize,
    iter: Peekable<Chars<'a>>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            line: 1,
            iter: source.chars().peekable(),
        }
    }

    fn identifier(&mut self) -> Option<Token> {
        let mut lexeme_builder = vec![];

        while let Some(&c) = self.iter.peek() {
            if !c.is_alphanumeric() {
                break;
            }
            lexeme_builder.push(c);
            self.iter.next();
        }

        let lexeme: String = lexeme_builder.into_iter().collect();
        Some(Token {
            kind: TokenType::Identifier,
            line: self.line,
            lexeme,
        })
    }

    fn number(&mut self) -> Option<Token> {
        let mut lexeme_builder = vec![];

        while let Some(&c) = self.iter.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            lexeme_builder.push(c);
            self.iter.next();
        }

        let peek_next = self.peek_next().take_if(|x| x.is_ascii_digit());
        let next = self.iter.peek();
        if next == Some(&'.') && peek_next.is_some() {
            lexeme_builder.push(*next?);
            self.iter.next(); // Consume the '.'
            while let Some(&c) = self.iter.peek() {
                if !c.is_ascii_digit() {
                    break;
                }
                lexeme_builder.push(c);
                self.iter.next();
            }
        }

        let lexeme: String = lexeme_builder.into_iter().collect();
        Some(Token {
            kind: TokenType::Number,
            line: self.line,
            lexeme,
        })
    }

    fn string(&mut self) -> Option<Token> {
        let mut lexeme_builder = vec![];
        while let Some(&c) = self.iter.peek() {
            if c == '"' {
                break;
            }

            if c == '\n' {
                self.line += 1;
            }

            lexeme_builder.push(c);
            self.iter.next();
        }

        if self.is_at_end() {
            return Some(Token {
                kind: TokenType::Error,
                lexeme: "Unterminated string.".into(),
                line: self.line,
            });
        }

        // Consume closing quote
        self.iter.next();

        let lexeme = lexeme_builder.into_iter().collect();
        Some(Token {
            kind: TokenType::String,
            lexeme,
            line: self.line,
        })
    }

    fn is_at_end(&mut self) -> bool {
        self.iter.peek().is_none()
    }

    fn peek_next(&mut self) -> Option<char> {
        self.iter.peek()?;
        let mut next_iter = self.iter.clone();
        next_iter.next();
        let next_c = next_iter.peek()?;
        Some(*next_c)
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.iter.peek() {
                None => return,
                Some(' ' | '\t' | '\r') => {
                    self.iter.next();
                }
                Some('\n') => {
                    self.line += 1;
                    self.iter.next();
                }
                Some('/') => {
                    if self.peek_next() == Some('/') {
                        while self.iter.peek() != Some(&'\n') && !self.is_at_end() {
                            self.iter.next();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        if self.is_at_end() {
            return Some(Token {
                kind: TokenType::Eof,
                lexeme: "".into(),
                line: self.line,
            });
        };
        let c = self.iter.peek().unwrap();
        if c.is_alphabetic() {
            return self.identifier();
        }
        if c.is_ascii_digit() {
            return self.number();
        }

        let mut token = Token {
            kind: TokenType::Error,
            lexeme: c.to_string(),
            line: self.line,
        };

        token.kind = match self.iter.next()? {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ';' => TokenType::Semicolon,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            '/' => TokenType::Slash,
            '*' => TokenType::Star,
            '!' => {
                if self.iter.next_if_eq(&'=').is_some() {
                    token.lexeme = "!=".into();
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            '=' => {
                if self.iter.next_if_eq(&'=').is_some() {
                    token.lexeme = "==".into();
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '<' => {
                if self.iter.next_if_eq(&'=').is_some() {
                    token.lexeme = "<=".into();
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.iter.next_if_eq(&'=').is_some() {
                    token.lexeme = ">=".into();
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '"' => {
                token = self.string()?;
                token.kind
            }
            _ => {
                token.lexeme = format!("Unexpected character '{}'", token.lexeme);
                TokenType::Error
            }
        };
        Some(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_scans_end_of_file() {
        let source = "";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token {
                kind: TokenType::Eof,
                line: 1,
                lexeme: "".into(),
            },
        );
    }

    #[test]
    fn it_skips_whitespace_and_comments() {
        let source = "    \t  \n // \r\n \t   ";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token {
                kind: TokenType::Eof,
                line: 3,
                lexeme: "".into(),
            },
        );
    }

    #[test]
    fn it_scans_an_identifier() {
        let source = "identifier\nidentifier1234";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token {
                kind: TokenType::Identifier,
                line: 1,
                lexeme: "identifier".into()
            }
        );
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token {
                kind: TokenType::Identifier,
                line: 2,
                lexeme: "identifier1234".into()
            }
        );
    }

    #[test]
    fn it_scans_a_number() {
        let source = "12345.6789\n54321";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token {
                kind: TokenType::Number,
                line: 1,
                lexeme: "12345.6789".into()
            }
        );

        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token {
                kind: TokenType::Number,
                line: 2,
                lexeme: "54321".into()
            }
        );
    }

    #[test]
    fn it_scans_single_characters() {
        let source = "(){};,.-+/*! = < > $";
        let mut scanner = Scanner::new(source);
        let expected_tokens = vec![
            Token {
                kind: TokenType::LeftParen,
                lexeme: "(".into(),
                line: 1,
            },
            Token {
                kind: TokenType::RightParen,
                lexeme: ")".into(),
                line: 1,
            },
            Token {
                kind: TokenType::LeftBrace,
                lexeme: "{".into(),
                line: 1,
            },
            Token {
                kind: TokenType::RightBrace,
                lexeme: "}".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Semicolon,
                lexeme: ";".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Comma,
                lexeme: ",".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Dot,
                lexeme: ".".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Minus,
                lexeme: "-".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Plus,
                lexeme: "+".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Slash,
                lexeme: "/".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Star,
                lexeme: "*".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Bang,
                lexeme: "!".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Equal,
                lexeme: "=".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Less,
                lexeme: "<".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Greater,
                lexeme: ">".into(),
                line: 1,
            },
            Token {
                kind: TokenType::Error,
                lexeme: "Unexpected character '$'".into(),
                line: 1,
            },
        ];
        for expected_token in expected_tokens {
            let token = scanner.next().unwrap();
            assert_eq!(token, expected_token);
        }
    }

    #[test]
    fn it_scans_double_tokens() {
        let source = "== <= >= !=";
        let mut scanner = Scanner::new(source);
        let expected_tokens = vec![
            Token {
                kind: TokenType::EqualEqual,
                lexeme: "==".into(),
                line: 1,
            },
            Token {
                kind: TokenType::LessEqual,
                lexeme: "<=".into(),
                line: 1,
            },
            Token {
                kind: TokenType::GreaterEqual,
                lexeme: ">=".into(),
                line: 1,
            },
            Token {
                kind: TokenType::BangEqual,
                lexeme: "!=".into(),
                line: 1,
            },
        ];

        for expected_token in expected_tokens {
            let token = scanner.next().unwrap();
            assert_eq!(token, expected_token);
        }
    }

    #[test]
    fn it_scans_a_string() {
        let source = "\"hello world\"";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token {
                kind: TokenType::String,
                lexeme: "hello world".into(),
                line: 1
            }
        );
    }

    #[test]
    fn it_reports_unterminated_string() {
        let source = "\"hello world";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token {
                kind: TokenType::Error,
                lexeme: "Unterminated string.".into(),
                line: 1
            }
        );
    }
}
