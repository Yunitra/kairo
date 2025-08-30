// SPDX-License-Identifier: Apache-2.0

//! 语法分析器模块入口
//!
//! 子模块拆分：
//! - `expressions.rs` 表达式解析
//! - `statements.rs` 语句解析
//! - `pattern.rs` 模式与 match 案例解析
//! - `types.rs` 类型解析
//! - `assignments.rs` 赋值/自增自减解析
//! - `functions.rs` 函数与返回解析

mod expressions;
mod statements;
mod pattern;
mod types;
mod assignments;
mod functions;

use crate::lexer::{Token, TokenType};
use crate::ast::Program;
use crate::error::{KairoError, Result};

/// 语法分析器：从 `Token` 序列解析为 AST `Program`
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// 创建新的 Parser 实例
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// 入口：解析为 `Program`
    pub fn parse(&mut self) -> Result<Program> {
        self.statements_parse_program()
    }

    // =============== 内部通用工具 ===============

    /// 到达输入末尾？
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || 
        matches!(self.tokens.get(self.current).map(|t| &t.token_type), Some(TokenType::Eof))
    }

    /// 当前 token
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    /// 前进一个 token
    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current.saturating_sub(1))
    }

    /// 判定当前 token 类型（仅比较判别式）
    fn check(&self, token_type: &TokenType) -> bool {
        if let Some(token) = self.current_token() {
            std::mem::discriminant(&token.token_type) == std::mem::discriminant(token_type)
        } else {
            false
        }
    }

    /// 消费一个指定类型的 token，否则报错
    fn consume(&mut self, expected: TokenType, message: &str) -> Result<Token> {
        if let Some(token) = self.current_token() {
            if std::mem::discriminant(&token.token_type) == std::mem::discriminant(&expected) {
                let token = token.clone();
                self.advance();
                return Ok(token);
            }
            return Err(KairoError::syntax(
                format!("{}, 但找到了 {:?}", message, token.token_type),
                token.line,
                token.column,
            ));
        }
        // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
            let prev_token = &self.tokens[self.current - 1];
            (prev_token.line, prev_token.column)
        } else {
            (1, 1)
        };
        Err(KairoError::syntax(message.to_string(), line, column))
    }

    /// 跳过换行（目前未直接使用，保留以备细粒度控制）
    #[allow(dead_code)]
    pub(crate) fn skip_newlines(&mut self) {
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Newline) {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// 跳过注释与换行
    fn skip_comments_and_newlines(&mut self) {
        while let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::Newline | 
                TokenType::SingleComment(_) | 
                TokenType::MultiComment(_) | 
                TokenType::DocComment(_) => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// 期望一个标识符
    fn expect_identifier(&mut self) -> Result<String> {
        if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type {
                let name = name.clone();
                self.advance();
                return Ok(name);
            }
            return Err(KairoError::syntax(
                format!("期望标识符, 但找到了 {:?}", token.token_type),
                token.line,
                token.column,
            ));
        }
        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
            let prev_token = &self.tokens[self.current - 1];
            (prev_token.line, prev_token.column)
        } else {
            (1, 1)
        };
        Err(KairoError::syntax("期望标识符".to_string(), line, column))
    }
}
