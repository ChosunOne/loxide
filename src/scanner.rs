use std::iter::Peekable;
use std::str::Chars;

use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct Scanner<'a> {
    line: usize,
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
        Some(Token::Identifier(TokenData {
            line: self.line,
            lexeme,
        }))
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
        Some(Token::Number(TokenData {
            line: self.line,
            lexeme,
        }))
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
            return Some(Token::Error(TokenData {
                lexeme: "Unterminated string.".into(),
                line: self.line,
            }));
        }

        // Consume closing quote
        self.iter.next();

        let lexeme = lexeme_builder.into_iter().collect();
        Some(Token::String(TokenData {
            lexeme,
            line: self.line,
        }))
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
            return Some(Token::Eof(TokenData {
                lexeme: "".into(),
                line: self.line,
            }));
        };
        let c = self.iter.peek().unwrap();
        if c.is_alphabetic() {
            return self.identifier();
        }
        if c.is_ascii_digit() {
            return self.number();
        }

        let token_data = TokenData {
            lexeme: c.to_string(),
            line: self.line,
        };

        match self.iter.next()? {
            '(' => Some(Token::LeftParen(token_data)),
            ')' => Some(Token::RightParen(token_data)),
            '{' => Some(Token::LeftBrace(token_data)),
            '}' => Some(Token::RightBrace(token_data)),
            ';' => Some(Token::Semicolon(token_data)),
            ',' => Some(Token::Comma(token_data)),
            '.' => Some(Token::Dot(token_data)),
            '-' => Some(Token::Minus(token_data)),
            '+' => Some(Token::Plus(token_data)),
            '/' => Some(Token::Slash(token_data)),
            '*' => Some(Token::Star(token_data)),
            '!' => {
                if self.iter.next_if_eq(&'=').is_some() {
                    Some(Token::BangEqual(TokenData {
                        lexeme: "!=".into(),
                        line: token_data.line,
                    }))
                } else {
                    Some(Token::Bang(token_data))
                }
            }
            '=' => {
                if self.iter.next_if_eq(&'=').is_some() {
                    Some(Token::EqualEqual(TokenData {
                        lexeme: "==".into(),
                        line: token_data.line,
                    }))
                } else {
                    Some(Token::Equal(token_data))
                }
            }
            '<' => {
                if self.iter.next_if_eq(&'=').is_some() {
                    Some(Token::LessEqual(TokenData {
                        lexeme: "<=".into(),
                        line: token_data.line,
                    }))
                } else {
                    Some(Token::Less(token_data))
                }
            }
            '>' => {
                if self.iter.next_if_eq(&'=').is_some() {
                    Some(Token::GreaterEqual(TokenData {
                        lexeme: ">=".into(),
                        line: token_data.line,
                    }))
                } else {
                    Some(Token::Greater(token_data))
                }
            }
            '"' => self.string(),
            _ => Some(Token::Error(TokenData {
                lexeme: format!("Unexpected character '{}'", token_data.lexeme),
                line: token_data.line,
            })),
        }
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
            Token::Eof(TokenData {
                line: 1,
                lexeme: "".into(),
            }),
        );
    }

    #[test]
    fn it_skips_whitespace_and_comments() {
        let source = "    \t  \n // \r\n \t   ";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token::Eof(TokenData {
                line: 3,
                lexeme: "".into(),
            }),
        );
    }

    #[test]
    fn it_scans_an_identifier() {
        let source = "identifier\nidentifier1234";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token::Identifier(TokenData {
                line: 1,
                lexeme: "identifier".into()
            })
        );
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token::Identifier(TokenData {
                line: 2,
                lexeme: "identifier1234".into()
            })
        );
    }

    #[test]
    fn it_scans_a_number() {
        let source = "12345.6789\n54321";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token::Number(TokenData {
                line: 1,
                lexeme: "12345.6789".into()
            })
        );

        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token::Number(TokenData {
                line: 2,
                lexeme: "54321".into()
            })
        );
    }

    #[test]
    fn it_scans_single_characters() {
        let source = "(){};,.-+/*! = < > $";
        let mut scanner = Scanner::new(source);
        let expected_tokens = vec![
            Token::LeftParen(TokenData {
                lexeme: "(".into(),
                line: 1,
            }),
            Token::RightParen(TokenData {
                lexeme: ")".into(),
                line: 1,
            }),
            Token::LeftBrace(TokenData {
                lexeme: "{".into(),
                line: 1,
            }),
            Token::RightBrace(TokenData {
                lexeme: "}".into(),
                line: 1,
            }),
            Token::Semicolon(TokenData {
                lexeme: ";".into(),
                line: 1,
            }),
            Token::Comma(TokenData {
                lexeme: ",".into(),
                line: 1,
            }),
            Token::Dot(TokenData {
                lexeme: ".".into(),
                line: 1,
            }),
            Token::Minus(TokenData {
                lexeme: "-".into(),
                line: 1,
            }),
            Token::Plus(TokenData {
                lexeme: "+".into(),
                line: 1,
            }),
            Token::Slash(TokenData {
                lexeme: "/".into(),
                line: 1,
            }),
            Token::Star(TokenData {
                lexeme: "*".into(),
                line: 1,
            }),
            Token::Bang(TokenData {
                lexeme: "!".into(),
                line: 1,
            }),
            Token::Equal(TokenData {
                lexeme: "=".into(),
                line: 1,
            }),
            Token::Less(TokenData {
                lexeme: "<".into(),
                line: 1,
            }),
            Token::Greater(TokenData {
                lexeme: ">".into(),
                line: 1,
            }),
            Token::Error(TokenData {
                lexeme: "Unexpected character '$'".into(),
                line: 1,
            }),
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
            Token::EqualEqual(TokenData {
                lexeme: "==".into(),
                line: 1,
            }),
            Token::LessEqual(TokenData {
                lexeme: "<=".into(),
                line: 1,
            }),
            Token::GreaterEqual(TokenData {
                lexeme: ">=".into(),
                line: 1,
            }),
            Token::BangEqual(TokenData {
                lexeme: "!=".into(),
                line: 1,
            }),
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
            Token::String(TokenData {
                lexeme: "hello world".into(),
                line: 1
            })
        );
    }

    #[test]
    fn it_reports_unterminated_string() {
        let source = "\"hello world";
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(
            token,
            Token::Error(TokenData {
                lexeme: "Unterminated string.".into(),
                line: 1
            })
        );
    }
}
