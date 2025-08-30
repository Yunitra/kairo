// SPDX-License-Identifier: Apache-2.0

//! 表达式解析

use crate::ast::{Expression, BinaryOperator, UnaryOperator};
use crate::error::{KairoError, Result};
use crate::lexer::TokenType;

use super::Parser;

impl Parser {
    pub(crate) fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_pipeline()
    }

    pub(crate) fn parse_pipeline(&mut self) -> Result<Expression> {
        let mut expr = self.parse_if_expression()?;

        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::PipeArrow) {
                let (line, column) = (token.line, token.column);
                self.advance();
                let right = self.parse_if_expression()?;
                expr = Expression::Pipeline {
                    left: Box::new(expr),
                    right: Box::new(right),
                    line,
                    column,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    pub(crate) fn parse_if_expression(&mut self) -> Result<Expression> {
        use crate::lexer::TokenType::*;
        if self.check(&If) {
            self.advance();
            let condition = self.parse_or()?;
            self.consume(LeftBrace, "期望 '{'")?;
            let then_expr = self.parse_expression()?;
            self.consume(RightBrace, "期望 '}'")?;
            self.consume(Else, "期望 'else'")?;
            let else_expr = if self.check(&If) {
                self.parse_if_expression()?
            } else {
                self.consume(LeftBrace, "期望 '{'")?;
                let expr = self.parse_expression()?;
                self.consume(RightBrace, "期望 '}'")?;
                expr
            };
            return Ok(Expression::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_expr),
                else_branch: Box::new(else_expr),
            });
        } else if self.check(&Match) {
            self.advance();
            let value = self.parse_or()?;
            self.consume(LeftBrace, "期望 '{'")?;
            let mut arms = Vec::new();
            while !self.check(&RightBrace) && !self.is_at_end() {
                self.skip_comments_and_newlines();
                if self.check(&RightBrace) { break; }
                let pattern = self.parse_pattern()?;
                let guard = if self.check(&If) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else { None };
                self.consume(Arrow, "期望 '->'")?;
                let arm_value = self.parse_expression()?;
                arms.push(crate::ast::MatchArmExpr { pattern, guard, value: arm_value });
                self.skip_comments_and_newlines();
            }
            self.consume(RightBrace, "期望 '}'")?;
            return Ok(Expression::MatchExpr { value: Box::new(value), arms });
        }
        self.parse_or()
    }

    pub(crate) fn parse_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and()?;
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Or) {
                self.advance();
                let right = self.parse_and()?;
                expr = Expression::Binary { left: Box::new(expr), operator: BinaryOperator::Or, right: Box::new(right) };
            } else { break; }
        }
        Ok(expr)
    }

    pub(crate) fn parse_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::And) {
                self.advance();
                let right = self.parse_equality()?;
                expr = Expression::Binary { left: Box::new(expr), operator: BinaryOperator::And, right: Box::new(right) };
            } else { break; }
        }
        Ok(expr)
    }

    pub(crate) fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;
        while let Some(token) = self.current_token() {
            let operator = match &token.token_type {
                TokenType::Equal => BinaryOperator::Equal,
                TokenType::NotEqual => BinaryOperator::NotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expression::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }
        Ok(expr)
    }

    pub(crate) fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_range()?;
        while let Some(token) = self.current_token() {
            let operator = match &token.token_type {
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_range()?;
            expr = Expression::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }
        Ok(expr)
    }

    pub(crate) fn parse_range(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;
        if let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::DotDot => {
                    self.advance();
                    let end = self.parse_term()?;
                    expr = Expression::Range { start: Box::new(expr), end: Box::new(end), inclusive: false };
                }
                TokenType::DotDotEqual => {
                    self.advance();
                    let end = self.parse_term()?;
                    expr = Expression::Range { start: Box::new(expr), end: Box::new(end), inclusive: true };
                }
                _ => {}
            }
        }
        Ok(expr)
    }

    pub(crate) fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;
        while let Some(token) = self.current_token() {
            let operator = match &token.token_type {
                TokenType::Minus => BinaryOperator::Subtract,
                TokenType::Plus => BinaryOperator::Add,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            expr = Expression::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }
        Ok(expr)
    }

    pub(crate) fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;
        while let Some(token) = self.current_token() {
            let operator = match &token.token_type {
                TokenType::Divide => BinaryOperator::Divide,
                TokenType::Multiply => BinaryOperator::Multiply,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = Expression::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }
        Ok(expr)
    }

    pub(crate) fn parse_unary(&mut self) -> Result<Expression> {
        if let Some(token) = self.current_token() {
            let operator = match &token.token_type {
                TokenType::Not => UnaryOperator::Not,
                TokenType::Minus => UnaryOperator::Minus,
                _ => return self.parse_call(),
            };
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary { operator, operand: Box::new(operand) });
        }
        self.parse_call()
    }

    pub(crate) fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_postfix()?;
        loop {
            if let Some(token) = self.current_token() {
                if matches!(token.token_type, TokenType::LeftParen) {
                    let (line, column) = (token.line, token.column);
                    self.advance();
                    let mut arguments = Vec::new();
                    if !self.check(&TokenType::RightParen) {
                        arguments.push(self.parse_expression()?);
                        while self.check(&TokenType::Comma) {
                            self.advance();
                            if self.check(&TokenType::RightParen) { break; }
                            arguments.push(self.parse_expression()?);
                        }
                    }
                    self.consume(TokenType::RightParen, "期望 ')'")?;
                    if let Expression::Identifier(name) = expr {
                        expr = Expression::FunctionCall { name, arguments, line, column };
                    } else {
                        return Err(KairoError::syntax("只能调用函数".to_string(), line, column));
                    }
                } else if matches!(token.token_type, TokenType::Dot) {
                    let (line, column) = (token.line, token.column);
                    self.advance();
                    let method_name = if let Some(token) = self.current_token() {
                        if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); self.advance(); name }
                        else { return Err(KairoError::syntax("期望方法名".to_string(), token.line, token.column)); }
                    } else { return Err(KairoError::syntax("期望方法名".to_string(), 1, 1)); };
                    self.consume(TokenType::LeftParen, "期望 '('")?;
                    let mut arguments = Vec::new();
                    if !self.check(&TokenType::RightParen) {
                        arguments.push(self.parse_expression()?);
                        while self.check(&TokenType::Comma) {
                            self.advance();
                            if self.check(&TokenType::RightParen) { break; }
                            arguments.push(self.parse_expression()?);
                        }
                    }
                    self.consume(TokenType::RightParen, "期望 ')'")?;
                    expr = Expression::MethodCall { object: Box::new(expr), method: method_name, arguments, line, column };
                } else { break; }
            } else { break; }
        }
        Ok(expr)
    }

    pub(crate) fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;
        if let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::Increment => {
                    if let Expression::Identifier(name) = expr {
                        let line = token.line; let column = token.column; self.advance();
                        expr = Expression::Assignment { target: name.clone(), operator: crate::ast::AssignmentOperator::AddAssign, value: Box::new(Expression::Literal(crate::types::KairoValue::Int(1))), line, column };
                    } else { return Err(KairoError::syntax("++ 只能用于变量".to_string(), token.line, token.column)); }
                }
                TokenType::Decrement => {
                    if let Expression::Identifier(name) = expr {
                        let line = token.line; let column = token.column; self.advance();
                        expr = Expression::Assignment { target: name.clone(), operator: crate::ast::AssignmentOperator::SubAssign, value: Box::new(Expression::Literal(crate::types::KairoValue::Int(1))), line, column };
                    } else { return Err(KairoError::syntax("-- 只能用于变量".to_string(), token.line, token.column)); }
                }
                _ => {}
            }
        }
        Ok(expr)
    }

    pub(crate) fn parse_primary(&mut self) -> Result<Expression> {
        if let Some(token) = self.current_token() {
            let (token_type, line, column) = (token.token_type.clone(), token.line, token.column);
            match &token_type {
                TokenType::BoolLiteral(value) => { let value = *value; self.advance(); Ok(Expression::Literal(crate::types::KairoValue::Bool(value))) }
                TokenType::IntLiteral(value) => { let value = *value; self.advance(); Ok(Expression::Literal(crate::types::KairoValue::Int(value))) }
                TokenType::FloatLiteral(value) => { let value = *value; self.advance(); Ok(Expression::Literal(crate::types::KairoValue::Float(value))) }
                TokenType::TextLiteral(value) => { let value = value.clone(); self.advance(); Ok(Expression::Literal(crate::types::KairoValue::Text(value))) }
                TokenType::Identifier(name) => { let name = name.clone(); self.advance(); Ok(Expression::Identifier(name)) }
                TokenType::Underscore => { self.advance(); Ok(Expression::Placeholder { line, column }) }
                TokenType::LeftParen => {
                    self.advance();
                    if self.check(&TokenType::RightParen) { self.advance(); return Ok(Expression::Tuple(Vec::new())); }
                    let first_expr = self.parse_expression()?;
                    if self.check(&TokenType::Comma) {
                        let mut elements = vec![first_expr];
                        while self.check(&TokenType::Comma) { self.advance(); if self.check(&TokenType::RightParen) { break; } elements.push(self.parse_expression()?); }
                        self.consume(TokenType::RightParen, "期望 ')'")?;
                        Ok(Expression::Tuple(elements))
                    } else {
                        self.consume(TokenType::RightParen, "期望 ')'")?;
                        Ok(first_expr)
                    }
                }
                TokenType::LeftBracket => {
                    self.advance();
                    let mut elements = Vec::new();
                    if !self.check(&TokenType::RightBracket) {
                        elements.push(self.parse_expression()?);
                        while self.check(&TokenType::Comma) { self.advance(); if self.check(&TokenType::RightBracket) { break; } elements.push(self.parse_expression()?); }
                    }
                    self.consume(TokenType::RightBracket, "期望 ']'")?;
                    Ok(Expression::List(elements))
                }
                TokenType::LeftBrace => {
                    self.advance();
                    let mut pairs = Vec::new();
                    if !self.check(&TokenType::RightBrace) {
                        let key = self.parse_expression()?;
                        self.consume(TokenType::Colon, "期望 ':'")?;
                        let value = self.parse_expression()?;
                        pairs.push((key, value));
                        while self.check(&TokenType::Comma) { self.advance(); if self.check(&TokenType::RightBrace) { break; } let key = self.parse_expression()?; self.consume(TokenType::Colon, "期望 ':'")?; let value = self.parse_expression()?; pairs.push((key, value)); }
                    }
                    self.consume(TokenType::RightBrace, "期望 '}'")?;
                    Ok(Expression::Map(pairs))
                }
                _ => Err(KairoError::syntax(format!("意外的 token: {:?}", token_type), line, column)),
            }
        } else {
            Err(KairoError::syntax("意外的文件结尾".to_string(), 1, 1))
        }
    }
}
