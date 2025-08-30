// SPDX-License-Identifier: Apache-2.0

//! 模式解析

use crate::ast::{Pattern, Expression};
use crate::ast::expression::ExpressionKind;
use crate::error::{KairoError, Result};
use crate::lexer::TokenType;
use crate::types::KairoValue;

use super::Parser;

impl Parser {
    pub(crate) fn parse_pattern(&mut self) -> Result<Pattern> {
        if let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::Underscore => { self.advance(); Ok(Pattern::Wildcard) }
                TokenType::Identifier(name) if name == "_" => { self.advance(); Ok(Pattern::Wildcard) }
                TokenType::Identifier(name) => {
                    let name = name.clone();
                    let (line, column) = (token.line, token.column);
                    self.advance();
                    if self.check(&TokenType::Colon) {
                        self.advance();
                        if let Some(type_token) = self.current_token() {
                            if let TokenType::Identifier(type_name) = &type_token.token_type {
                                let type_name = type_name.clone();
                                self.advance();
                                return Ok(Pattern::TypePattern { var: name, type_name });
                            }
                        }
                        return Err(KairoError::syntax("期望类型名".to_string(), line, column));
                    }
                    Ok(Pattern::Identifier(name))
                }
                TokenType::IntLiteral(value) => { let value = *value; self.advance(); Ok(Pattern::Literal(KairoValue::Int(value))) }
                TokenType::FloatLiteral(value) => { let value = *value; self.advance(); Ok(Pattern::Literal(KairoValue::Float(value))) }
                TokenType::TextLiteral(value) => { let value = value.clone(); self.advance(); Ok(Pattern::Literal(KairoValue::Text(value))) }
                TokenType::BoolLiteral(value) => { let value = *value; self.advance(); Ok(Pattern::Literal(KairoValue::Bool(value))) }
                TokenType::In => {
                    let (line, column) = (token.line, token.column);
                    self.advance();
                    if self.check(&TokenType::DotDot) || self.check(&TokenType::DotDotEqual) {
                        let inclusive = if self.check(&TokenType::DotDotEqual) { true } else { false };
                        self.advance();
                        let end = self.parse_expression()?;
                        let start = Expression { kind: ExpressionKind::Literal(KairoValue::Int(0)), line, column };
                        let range = Expression { kind: ExpressionKind::Range { start: Box::new(start), end: Box::new(end), inclusive }, line, column };
                        Ok(Pattern::In(range))
                    } else {
                        let expr = self.parse_expression()?;
                        Ok(Pattern::In(expr))
                    }
                }
                TokenType::LeftParen => {
                    self.advance();
                    let mut patterns = Vec::new();
                    if !self.check(&TokenType::RightParen) {
                        patterns.push(self.parse_pattern()?);
                        while self.check(&TokenType::Comma) {
                            self.advance();
                            if self.check(&TokenType::RightParen) { break; }
                            patterns.push(self.parse_pattern()?);
                        }
                    }
                    self.consume(TokenType::RightParen, "期望 ')'")?;
                    Ok(Pattern::Tuple(patterns))
                }
                _ => Err(KairoError::syntax(format!("无效的模式: {:?}", token.token_type), token.line, token.column)),
            }
        } else {
            // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
            let prev_token = &self.tokens[self.current - 1];
            (prev_token.line, prev_token.column)
        } else {
            (1, 1)
        };
        Err(KairoError::syntax("期望模式".to_string(), line, column))
        }
    }
}
