// SPDX-License-Identifier: Apache-2.0

//! 类型解析

use crate::error::{KairoError, Result};
use crate::lexer::TokenType;
use crate::types::KairoType;

use super::Parser;

impl Parser {
    pub(crate) fn parse_type(&mut self) -> Result<KairoType> {
        let mut base_type = self.parse_base_type()?;
        
        // 检查是否有 ? 表示可空类型
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Question) {
                self.advance();
                base_type = KairoType::Nullable(Box::new(base_type));
            }
        }
        
        Ok(base_type)
    }
    
    fn parse_base_type(&mut self) -> Result<KairoType> {
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
                            let inner_type = self.parse_base_type()?;
                            self.consume(TokenType::RightBracket, "期望 ']'")?;
                            Ok(KairoType::List(Box::new(inner_type)))
                        }
                        "Map" => {
                            self.consume(TokenType::LeftBracket, "期望 '['")?;
                            let key_type = self.parse_base_type()?;
                            self.consume(TokenType::Arrow, "期望 '->'")?;
                            let value_type = self.parse_base_type()?;
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
                        types.push(self.parse_base_type()?);
                        while self.check(&TokenType::Comma) {
                            self.advance();
                            if self.check(&TokenType::RightParen) { break; }
                            types.push(self.parse_base_type()?);
                        }
                    }
                    self.consume(TokenType::RightParen, "期望 ')'")?;
                    Ok(KairoType::Tuple(types))
                }
                _ => Err(KairoError::syntax("期望类型名".to_string(), line, column)),
            }
        } else {
            // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
            let prev_token = &self.tokens[self.current - 1];
            (prev_token.line, prev_token.column)
        } else {
            (1, 1)
        };
        Err(KairoError::syntax("期望类型名".to_string(), line, column))
        }
    }
}
