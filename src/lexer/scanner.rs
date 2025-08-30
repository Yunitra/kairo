// SPDX-License-Identifier: Apache-2.0

//! 词法扫描主逻辑：`next_token` 与 `tokenize`

use crate::error::{KairoError, Result};

use super::token::{Token, TokenType};

impl super::Lexer {
    pub fn next_token(&mut self) -> Result<Token> {
        loop {
            self.skip_whitespace();

            let line = self.line;
            let column = self.column;

            match self.current_char {
                None => return Ok(Token { token_type: TokenType::Eof, line, column }),
                
                Some('\n') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::Newline, line, column });
                }

                Some('#') => {
                    let comment = self.read_single_comment();
                    return Ok(Token { 
                        token_type: TokenType::SingleComment(comment), 
                        line, 
                        column 
                    });
                }

                Some('-') if self.peek() == Some('-') => {
                    let comment = self.read_multi_comment()?;
                    return Ok(Token { 
                        token_type: TokenType::MultiComment(comment), 
                        line, 
                        column 
                    });
                }

                Some('*') if self.peek() == Some('*') => {
                    let comment = self.read_doc_comment();
                    return Ok(Token { 
                        token_type: TokenType::DocComment(comment), 
                        line, 
                        column 
                    });
                }

                Some('"') => {
                    let string = self.read_string('"')?;
                    return Ok(Token { 
                        token_type: TokenType::TextLiteral(string), 
                        line, 
                        column 
                    });
                }

                Some('\'') => {
                    let string = self.read_string('\'')?;
                    return Ok(Token { 
                        token_type: TokenType::TextLiteral(string), 
                        line, 
                        column 
                    });
                }

                Some(ch) if ch.is_ascii_digit() => {
                    let number = self.read_number();
                    return Ok(Token { token_type: number, line, column });
                }

                Some('_') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::Underscore, line, column });
                }

                Some('$') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::Dollar, line, column });
                }

                Some('?') => {
                    self.advance();
                    if self.current_char == Some('.') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::SafeCall, line, column });
                    } else if self.current_char == Some(':') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::Elvis, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Question, line, column });
                }

                Some(ch) if ch.is_alphabetic() => {
                    let token_type = self.keyword_or_identifier();
                    return Ok(Token { token_type, line, column });
                }

                Some('=') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::Equal, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Assign, line, column });
                }

                Some('!') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::NotEqual, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Exclamation, line, column });
                }

                Some('<') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::LessEqual, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Less, line, column });
                }

                Some('>') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::GreaterEqual, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Greater, line, column });
                }

                Some('-') => {
                    self.advance();
                    if self.current_char == Some('>') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::Arrow, line, column });
                    } else if self.current_char == Some('-') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::Decrement, line, column });
                    } else if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::MinusAssign, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Minus, line, column });
                }

                Some('+') => {
                    self.advance();
                    if self.current_char == Some('+') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::Increment, line, column });
                    } else if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::PlusAssign, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Plus, line, column });
                }

                Some('*') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::MultiplyAssign, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Multiply, line, column });
                }

                Some('/') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::DivideAssign, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Divide, line, column });
                }

                Some('(') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::LeftParen, line, column });
                }

                Some(')') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::RightParen, line, column });
                }

                Some('[') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::LeftBracket, line, column });
                }

                Some(']') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::RightBracket, line, column });
                }

                Some('{') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::LeftBrace, line, column });
                }

                Some('}') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::RightBrace, line, column });
                }

                Some(',') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::Comma, line, column });
                }

                Some(':') => {
                    self.advance();
                    return Ok(Token { token_type: TokenType::Colon, line, column });
                }

                Some('.') => {
                    self.advance();
                    if self.current_char == Some('.') {
                        self.advance();
                        if self.current_char == Some('.') {
                            self.advance();
                            return Ok(Token { token_type: TokenType::Ellipsis, line, column });
                        }
                        if self.current_char == Some('=') {
                            self.advance();
                            return Ok(Token { token_type: TokenType::DotDotEqual, line, column });
                        }
                        return Ok(Token { token_type: TokenType::DotDot, line, column });
                    }
                    return Ok(Token { token_type: TokenType::Dot, line, column });
                }

                Some('|') => {
                    self.advance();
                    if self.current_char == Some('>') {
                        self.advance();
                        return Ok(Token { token_type: TokenType::PipeArrow, line, column });
                    }
                    // 如果没有 '>' 紧跟着，这是一个错误
                    return Err(KairoError::lexical(
                        "单独的管道符号不被支持，您是否想要使用管道操作符 |>?".to_string(),
                        line,
                        column,
                    ));
                }

                Some(ch) => {
                    return Err(KairoError::lexical(
                        format!("无效字符: {}", ch),
                        line,
                        column,
                    ));
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
}
