// SPDX-License-Identifier: Apache-2.0

//! 类型解析

use crate::error::{KairoError, Result};
use crate::lexer::TokenType;
use crate::types::KairoType;

use super::Parser;

impl Parser {
    pub(crate) fn parse_type(&mut self) -> Result<KairoType> {
        if let Some(token) = self.current_token() {
            let (token_type, line, column) = (token.token_type.clone(), token.line, token.column);
            match &token_type {
                TokenType::Identifier(type_name) => {
                    let type_name = type_name.clone();
                    self.advance();
                    match type_name.as_str() {
                        "Int" => Ok(KairoType::Int),
                        "Float" => Ok(KairoType::Float),
                        "Text" => Ok(KairoType::Text),
                        "Bool" => Ok(KairoType::Bool),
                        "Unit" => Ok(KairoType::Unit),
                        "List" => {
                            self.consume(TokenType::LeftBracket, "期望 '['")?;
                            let inner_type = self.parse_type()?;
                            self.consume(TokenType::RightBracket, "期望 ']'")?;
                            Ok(KairoType::List(Box::new(inner_type)))
                        }
                        "Map" => {
                            self.consume(TokenType::LeftBracket, "期望 '['")?;
                            let key_type = self.parse_type()?;
                            self.consume(TokenType::Arrow, "期望 '->'")?;
                            let value_type = self.parse_type()?;
                            self.consume(TokenType::RightBracket, "期望 ']'")?;
                            Ok(KairoType::Map(Box::new(key_type), Box::new(value_type)))
                        }
                        _ => Err(KairoError::syntax(format!("未知类型: {}", type_name), line, column)),
                    }
                }
                TokenType::Void => { self.advance(); Ok(KairoType::Void) }
                TokenType::LeftParen => {
                    self.advance();
                    let mut types = Vec::new();
                    if !self.check(&TokenType::RightParen) {
                        types.push(self.parse_type()?);
                        while self.check(&TokenType::Comma) {
                            self.advance();
                            if self.check(&TokenType::RightParen) { break; }
                            types.push(self.parse_type()?);
                        }
                    }
                    self.consume(TokenType::RightParen, "期望 ')'")?;
                    Ok(KairoType::Tuple(types))
                }
                _ => Err(KairoError::syntax("期望类型名".to_string(), line, column)),
            }
        } else {
            Err(KairoError::syntax("期望类型名".to_string(), 1, 1))
        }
    }
}
