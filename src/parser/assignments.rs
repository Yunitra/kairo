// SPDX-License-Identifier: Apache-2.0

//! 赋值与自增自减解析

use crate::ast::{AssignmentOperator, Expression};
use crate::ast::expression::{ExpressionKind};
use crate::error::{KairoError, Result};
use crate::lexer::TokenType;

use super::Parser;

impl Parser {
    pub(crate) fn parse_assignment_statement(&mut self) -> Result<crate::ast::Statement> {
        let (target, id_line, id_column) = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type {
                let name = name.clone();
                let id_line = token.line;
                let id_column = token.column;
                self.advance();
                (name, id_line, id_column)
            } else {
                return Err(KairoError::syntax(
                    "期望变量名".to_string(),
                    token.line,
                    token.column,
                ));
            }
        } else {
            // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
            let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                let prev_token = &self.tokens[self.current - 1];
                (prev_token.line, prev_token.column)
            } else {
                (1, 1)
            };
            return Err(KairoError::syntax("期望变量名".to_string(), line, column));
        };
        
        let (operator, op_line, _op_column) = if let Some(token) = self.current_token() {
            let op = match &token.token_type {
                TokenType::PlusAssign => AssignmentOperator::AddAssign,
                TokenType::MinusAssign => AssignmentOperator::SubAssign,
                TokenType::MultiplyAssign => AssignmentOperator::MulAssign,
                TokenType::DivideAssign => AssignmentOperator::DivAssign,
                _ => return Err(KairoError::syntax(
                    "期望赋值操作符".to_string(),
                    token.line,
                    token.column,
                )),
            };
            let op_line = token.line;
            let op_column = token.column;
            self.advance();
            (op, op_line, op_column)
        } else {
            // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
            let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                let prev_token = &self.tokens[self.current - 1];
                (prev_token.line, prev_token.column)
            } else {
                (1, 1)
            };
            return Err(KairoError::syntax("期望赋值操作符".to_string(), line, column));
        };
        
        let value = self.parse_expression()?;
        
        Ok(crate::ast::Statement { kind: crate::ast::StatementKind::Expression(Expression {
            kind: ExpressionKind::Assignment { target, operator, value: Box::new(value) },
            line: id_line.min(op_line),
            column: id_column,
        }), line: id_line.min(op_line), column: id_column })
    }
}
