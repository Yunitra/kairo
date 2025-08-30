// SPDX-License-Identifier: Apache-2.0

//! 表达式解析

use crate::ast::{Expression, BinaryOperator, UnaryOperator, ErrorHandlerClause, ErrorHandler, CatchClause};
use crate::ast::expression::ExpressionKind;
use crate::error::{KairoError, Result};
use crate::lexer::TokenType;

use super::Parser;

impl Parser {
    pub(crate) fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_pipeline()
    }

    pub(crate) fn parse_pipeline(&mut self) -> Result<Expression> {
        let mut expr = self.parse_ternary()?;

        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::PipeArrow) {
                let (line, column) = (token.line, token.column);
                self.advance();
                let right = self.parse_ternary()?;
                expr = Expression::new(ExpressionKind::Pipeline { left: Box::new(expr), right: Box::new(right) }, line, column);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    pub(crate) fn parse_ternary(&mut self) -> Result<Expression> {
        let expr = self.parse_elvis()?;
        
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Question) {
                // 先检查是否是Elvis操作符 ?:
                if let Some(next_token) = self.tokens.get(self.current + 1) {
                    if matches!(next_token.token_type, TokenType::Colon) {
                        // 这是Elvis操作符，不是三目运算符
                        return Ok(expr);
                    }
                }
                
                // 这是三目运算符
                let (line, column) = (token.line, token.column);
                self.advance();
                let then_expr = self.parse_elvis()?;
                self.consume(TokenType::Colon, "期望 ':'")?;
                let else_expr = self.parse_elvis()?;
                return Ok(Expression::new(ExpressionKind::Ternary { condition: Box::new(expr), then_expr: Box::new(then_expr), else_expr: Box::new(else_expr) }, line, column));
            }
        }
        
        Ok(expr)
    }

    pub(crate) fn parse_elvis(&mut self) -> Result<Expression> {
        let mut expr = self.parse_if_expression()?;
        
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Elvis) {
                let (line, column) = (token.line, token.column);
                self.advance();
                let right = self.parse_if_expression()?;
                expr = Expression::new(ExpressionKind::Elvis { left: Box::new(expr), right: Box::new(right) }, line, column);
            } else if matches!(token.token_type, TokenType::ErrorHandle) {
                // 错误处理语法糖 expr !: handler
                let (line, column) = (token.line, token.column);
                self.advance();

                let handler = if self.check(&TokenType::LeftBrace) {
                    self.parse_error_handler_match_block()?
                } else {
                    let handler_expr = self.parse_if_expression()?;
                    ErrorHandler::Simple(Box::new(handler_expr))
                };
                
                expr = Expression::new(ExpressionKind::ErrorHandle { expression: Box::new(expr), handler }, line, column);
            } else if matches!(token.token_type, TokenType::Question) {
                // 检查下一个token是否是冒号，如果是则为Elvis操作符
                if let Some(next_token) = self.tokens.get(self.current + 1) {
                    if matches!(next_token.token_type, TokenType::Colon) {
                        let (line, column) = (token.line, token.column);
                        self.advance(); // 跳过 ?
                        self.advance(); // 跳过 :
                        let right = self.parse_if_expression()?;
                        expr = Expression::new(ExpressionKind::Elvis { left: Box::new(expr), right: Box::new(right) }, line, column);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    pub(crate) fn parse_if_expression(&mut self) -> Result<Expression> {
        use crate::lexer::TokenType::*;
        if self.check(&If) {
            let if_token = self.advance().unwrap().clone();
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
            return Ok(Expression::new(ExpressionKind::If { condition: Box::new(condition), then_branch: Box::new(then_expr), else_branch: Box::new(else_expr) }, if_token.line, if_token.column));
        } else if self.check(&Match) {
            let match_token = self.advance().unwrap().clone();
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
            return Ok(Expression::new(ExpressionKind::MatchExpr { value: Box::new(value), arms }, match_token.line, match_token.column));
        }
        self.parse_or()
    }

    fn parse_error_handler_match_block(&mut self) -> Result<ErrorHandler> {
        self.consume(TokenType::LeftBrace, "期望 '{'")?;
        self.skip_comments_and_newlines();
    
        let mut clauses = Vec::<ErrorHandlerClause>::new();
        let mut default = None;
    
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let error_type = if self.check(&TokenType::Underscore) {
                self.advance();
                "_".to_string()
            } else {
                self.expect_identifier()?
            };
    
            // todo: support pattern matching like `NetworkError(code)`
    
            self.consume(TokenType::Arrow, "期望 '->'")?;
            let handler = self.parse_expression()?;
    
            if error_type == "_" {
                default = Some(Box::new(handler));
            } else {
                clauses.push(ErrorHandlerClause {
                    error_type,
                    variable: None, // Simplified for now
                    handler: Box::new(handler),
                });
            }
    
            if self.check(&TokenType::Comma) {
                self.advance();
            }
            self.skip_comments_and_newlines();
        }
    
        self.consume(TokenType::RightBrace, "期望 '}'")?;
        Ok(ErrorHandler::Match { clauses, default })
    }
    
    pub(crate) fn parse_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and()?;
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Or) {
                let op = self.advance().unwrap().clone();
                let right = self.parse_and()?;
                expr = Expression::new(ExpressionKind::Binary { left: Box::new(expr), operator: BinaryOperator::Or, right: Box::new(right) }, op.line, op.column);
            } else { break; }
        }
        Ok(expr)
    }

    pub(crate) fn parse_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::And) {
                let op = self.advance().unwrap().clone();
                let right = self.parse_equality()?;
                expr = Expression::new(ExpressionKind::Binary { left: Box::new(expr), operator: BinaryOperator::And, right: Box::new(right) }, op.line, op.column);
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
            let op = self.advance().unwrap().clone();
            let right = self.parse_comparison()?;
            expr = Expression::new(ExpressionKind::Binary { left: Box::new(expr), operator, right: Box::new(right) }, op.line, op.column);
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
            let op = self.advance().unwrap().clone();
            let right = self.parse_range()?;
            expr = Expression::new(ExpressionKind::Binary { left: Box::new(expr), operator, right: Box::new(right) }, op.line, op.column);
        }
        Ok(expr)
    }

    pub(crate) fn parse_range(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;
        if let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::DotDot => {
                    let op = self.advance().unwrap().clone();
                    let end = self.parse_term()?;
                    expr = Expression::new(ExpressionKind::Range { start: Box::new(expr), end: Box::new(end), inclusive: false }, op.line, op.column);
                }
                TokenType::DotDotEqual => {
                    let op = self.advance().unwrap().clone();
                    let end = self.parse_term()?;
                    expr = Expression::new(ExpressionKind::Range { start: Box::new(expr), end: Box::new(end), inclusive: true }, op.line, op.column);
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
            let op = self.advance().unwrap().clone();
            let right = self.parse_factor()?;
            expr = Expression::new(ExpressionKind::Binary { left: Box::new(expr), operator, right: Box::new(right) }, op.line, op.column);
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
            let op = self.advance().unwrap().clone();
            let right = self.parse_unary()?;
            expr = Expression::new(ExpressionKind::Binary { left: Box::new(expr), operator, right: Box::new(right) }, op.line, op.column);
        }
        Ok(expr)
    }

    pub(crate) fn parse_unary(&mut self) -> Result<Expression> {
        if let Some(token) = self.current_token() {
            let operator = match &token.token_type {
                TokenType::Not => UnaryOperator::Not,
                TokenType::Minus => UnaryOperator::Minus,
                _ => return self.parse_postfix(),
            };
            let op = self.advance().unwrap().clone();
            let operand = self.parse_unary()?;
            return Ok(Expression::new(ExpressionKind::Unary { operator, operand: Box::new(operand) }, op.line, op.column));
        }
        self.parse_postfix()
    }
    
    pub(crate) fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_call()?;
        loop {
            if let Some(token) = self.current_token() {
                match &token.token_type {
                    TokenType::Increment => {
                        if let Expression { kind: ExpressionKind::Identifier(name), .. } = expr {
                            let line = token.line; let column = token.column; self.advance();
                            expr = Expression::new(ExpressionKind::Assignment { target: name.clone(), operator: crate::ast::AssignmentOperator::AddAssign, value: Box::new(Expression::new(ExpressionKind::Literal(crate::types::KairoValue::Int(1)), line, column)) }, line, column);
                        } else { return Err(KairoError::syntax("++ 只能用于变量".to_string(), token.line, token.column)); }
                    }
                    TokenType::Decrement => {
                        if let Expression { kind: ExpressionKind::Identifier(name), .. } = expr {
                            let line = token.line; let column = token.column; self.advance();
                            expr = Expression::new(ExpressionKind::Assignment { target: name.clone(), operator: crate::ast::AssignmentOperator::SubAssign, value: Box::new(Expression::new(ExpressionKind::Literal(crate::types::KairoValue::Int(1)), line, column)) }, line, column);
                        } else { return Err(KairoError::syntax("-- 只能用于变量".to_string(), token.line, token.column)); }
                    }
                    TokenType::Exclamation => {
                        // 错误传播操作符 expr!
                        let line = token.line; 
                        let column = token.column; 
                        self.advance();
                        expr = Expression::new(ExpressionKind::ErrorPropagation { expression: Box::new(expr) }, line, column);
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    pub(crate) fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;
        loop {
            if let Some(token) = self.current_token() {
                if matches!(token.token_type, TokenType::LeftParen) {
                    let call_tok = self.advance().unwrap().clone();
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
                    if let Expression { kind: ExpressionKind::Identifier(name), .. } = expr {
                        expr = Expression::new(ExpressionKind::FunctionCall { name, arguments }, call_tok.line, call_tok.column);
                    } else {
                        return Err(KairoError::syntax("只能调用函数".to_string(), call_tok.line, call_tok.column));
                    }
                } else if matches!(token.token_type, TokenType::Dot) {
                    let dot_tok = self.advance().unwrap().clone();
                    let method_name = if let Some(token) = self.current_token() {
                        if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); self.advance(); name }
                        else { return Err(KairoError::syntax("期望方法名".to_string(), token.line, token.column)); }
                    } else { 
                        // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
                        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                            let prev_token = &self.tokens[self.current - 1];
                            (prev_token.line, prev_token.column)
                        } else {
                            (1, 1)
                        };
                        return Err(KairoError::syntax("期望方法名".to_string(), line, column)); 
                    };
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
                    expr = Expression::new(ExpressionKind::MethodCall { object: Box::new(expr), method: method_name, arguments }, dot_tok.line, dot_tok.column);
                } else if matches!(token.token_type, TokenType::SafeCall) {
                    let safe_tok = self.advance().unwrap().clone();
                    let method_name = if let Some(token) = self.current_token() {
                        if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); self.advance(); name }
                        else { return Err(KairoError::syntax("期望方法名".to_string(), token.line, token.column)); }
                    } else { 
                        // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
                        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                            let prev_token = &self.tokens[self.current - 1];
                            (prev_token.line, prev_token.column)
                        } else {
                            (1, 1)
                        };
                        return Err(KairoError::syntax("期望方法名".to_string(), line, column)); 
                    };
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
                    expr = Expression::new(ExpressionKind::SafeCall { object: Box::new(expr), method: method_name, arguments }, safe_tok.line, safe_tok.column);
                } else { break; }
            } else { break; }
        }
        Ok(expr)
    }

    pub(crate) fn parse_primary(&mut self) -> Result<Expression> {
        if let Some(token) = self.current_token() {
            let (token_type, line, column) = (token.token_type.clone(), token.line, token.column);
            match &token_type {
                TokenType::BoolLiteral(value) => { let value = *value; self.advance(); Ok(Expression::new(ExpressionKind::Literal(crate::types::KairoValue::Bool(value)), line, column)) }
                TokenType::IntLiteral(value) => { let value = *value; self.advance(); Ok(Expression::new(ExpressionKind::Literal(crate::types::KairoValue::Int(value)), line, column)) }
                TokenType::FloatLiteral(value) => { let value = *value; self.advance(); Ok(Expression::new(ExpressionKind::Literal(crate::types::KairoValue::Float(value)), line, column)) }
                TokenType::TextLiteral(value) => { let value = value.clone(); self.advance(); Ok(Expression::new(ExpressionKind::Literal(crate::types::KairoValue::Text(value)), line, column)) }
                TokenType::Identifier(name) => { let name = name.clone(); self.advance(); Ok(Expression::new(ExpressionKind::Identifier(name), line, column)) }
                TokenType::Underscore => { self.advance(); Ok(Expression::new(ExpressionKind::Placeholder, line, column)) }
                TokenType::Null => { self.advance(); Ok(Expression::new(ExpressionKind::Literal(crate::types::KairoValue::Null), line, column)) }
                TokenType::LeftParen => {
                    self.advance();
                    if self.check(&TokenType::RightParen) { self.advance(); return Ok(Expression::new(ExpressionKind::Tuple(Vec::new()), line, column)); }
                    let first_expr = self.parse_expression()?;
                    if self.check(&TokenType::Comma) {
                        let mut elements = vec![first_expr];
                        while self.check(&TokenType::Comma) { self.advance(); if self.check(&TokenType::RightParen) { break; } elements.push(self.parse_expression()?); }
                        self.consume(TokenType::RightParen, "期望 ')'")?;
                        Ok(Expression::new(ExpressionKind::Tuple(elements), line, column))
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
                    Ok(Expression::new(ExpressionKind::List(elements), line, column))
                }
                TokenType::LeftBrace => {
                    self.advance();
                    let mut pairs = Vec::new();
                    if !self.check(&TokenType::RightBrace) {
                        self.skip_comments_and_newlines();
                        let key = self.parse_expression()?;
                        self.consume(TokenType::Colon, "期望 ':'")?;
                        let value = self.parse_expression()?;
                        pairs.push((key, value));
                        while self.check(&TokenType::Comma) { 
                            self.advance(); 
                            self.skip_comments_and_newlines();
                            if self.check(&TokenType::RightBrace) { break; } 
                            let key = self.parse_expression()?; 
                            self.consume(TokenType::Colon, "期望 ':'")?; 
                            let value = self.parse_expression()?; 
                            pairs.push((key, value)); 
                        }
                    }
                    self.consume(TokenType::RightBrace, "期望 '}'")?;
                    Ok(Expression::new(ExpressionKind::Map(pairs), line, column))
                }
                TokenType::Try => {
                    self.advance();
                    
                    // 解析 try 表达式
                    let try_expr = if self.check(&TokenType::LeftBrace) {
                        let block = self.parse_block()?;
                        Expression::new(ExpressionKind::BlockExpr(block), line, column)
                    } else {
                        self.parse_expression()?
                    };
                    
                    let mut catch_clauses = Vec::new();
                    let mut default_catch = None;
                    
                    // 解析 catch 子句
                    while self.check(&TokenType::Catch) {
                        self.advance();
                        
                        if let Some(token) = self.current_token() {
                            if let TokenType::Identifier(error_type) = &token.token_type {
                                let error_type = error_type.clone();
                                self.advance();
                                
                                let variable = if self.check(&TokenType::Identifier("as".to_string())) {
                                    self.advance();
                                    if let Some(token) = self.current_token() {
                                        if let TokenType::Identifier(var_name) = &token.token_type {
                                            let var_name = var_name.clone();
                                            self.advance();
                                            Some(var_name)
                                        } else {
                                            return Err(KairoError::syntax(
                                                "期望变量名".to_string(),
                                                token.line,
                                                token.column,
                                            ));
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                
                                let handler = if self.check(&TokenType::LeftBrace) {
                                    let block = self.parse_block()?;
                                    Expression::new(ExpressionKind::BlockExpr(block), line, column)
                                } else {
                                    self.parse_expression()?
                                };
                                
                                catch_clauses.push(CatchClause {
                                    error_type,
                                    variable,
                                    handler: Box::new(handler),
                                });
                            } else if let TokenType::LeftBrace = &token.token_type {
                                // catch { ... } (默认捕获)
                                let block = self.parse_block()?;
                                default_catch = Some(Box::new(Expression::new(ExpressionKind::BlockExpr(block), line, column)));
                            } else {
                                // catch expr (默认捕获)
                                let handler = self.parse_expression()?;
                                default_catch = Some(Box::new(handler));
                            }
                        }
                    }
                    
                    Ok(Expression::new(ExpressionKind::TryCatch {
                        try_expr: Box::new(try_expr),
                        catch_clauses,
                        default_catch,
                    }, line, column))
                }
                _ => Err(KairoError::syntax(format!("意外的 token: {:?}", token_type), line, column)),
            }
        } else {
            // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
            let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                let prev_token = &self.tokens[self.current - 1];
                (prev_token.line, prev_token.column)
            } else {
                (1, 1)
            };
            Err(KairoError::syntax("意外的文件结尾".to_string(), line, column))
        }
    }
}
