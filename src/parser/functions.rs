// SPDX-License-Identifier: Apache-2.0

//! 函数声明与返回语句解析（含扩展函数）

use crate::ast::{Parameter, Statement, StatementKind};
use crate::error::{KairoError, Result};
use crate::lexer::TokenType;
use crate::types::KairoType;

use super::Parser;

impl Parser {
    pub(crate) fn parse_function_declaration(&mut self) -> Result<Statement> {
        self.consume(TokenType::Fun, "期望 'fun'")?;
        // 扩展函数或普通函数
        if let Some(token) = self.current_token() {
            if let TokenType::Identifier(type_name) = &token.token_type {
                let type_name = type_name.clone();
                self.advance();
                let full_type_name = if type_name == "List" && self.check(&TokenType::LeftBracket) {
                    self.advance();
                    let generic_param = if let Some(token) = self.current_token() {
                        if let TokenType::Identifier(param) = &token.token_type {
                            let param = param.clone();
                            self.advance();
                            param
                        } else { return Err(KairoError::syntax("期望泛型参数".to_string(), token.line, token.column)); }
                    } else { 
                        // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
                        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                            let prev_token = &self.tokens[self.current - 1];
                            (prev_token.line, prev_token.column)
                        } else {
                            (1, 1)
                        };
                        return Err(KairoError::syntax("期望泛型参数".to_string(), line, column)); 
                    };
                    self.consume(TokenType::RightBracket, "期望 ']'")?;
                    format!("List[{}]", generic_param)
                } else { type_name };
                if self.check(&TokenType::Dot) {
                    self.advance();
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
                    return self.parse_extension_function_body(full_type_name, method_name);
                } else {
                    let name = full_type_name;
                    return self.parse_regular_function_body(name);
                }
            }
        }
        // 普通函数名
        let name = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); self.advance(); name }
            else { return Err(KairoError::syntax("期望函数名".to_string(), token.line, token.column)); }
        } else { 
            // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
            let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                let prev_token = &self.tokens[self.current - 1];
                (prev_token.line, prev_token.column)
            } else {
                (1, 1)
            };
            return Err(KairoError::syntax("期望函数名".to_string(), line, column)); 
        };
        self.parse_regular_function_body(name)
    }

    pub(crate) fn parse_extension_function_body(&mut self, type_name: String, method_name: String) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "期望 '('")?;
        let mut parameters = Vec::new();
        // 自动注入 this 参数
        let this_param_type = if type_name.starts_with("List") {
            if type_name.contains('[') && type_name.contains(']') {
                let start = type_name.find('[').unwrap();
                let end = type_name.find(']').unwrap();
                let generic_param = &type_name[start + 1..end];
                let inner_type = match generic_param {
                    "T" => KairoType::Generic("T".to_string()),
                    "Int" => KairoType::Int,
                    "Float" => KairoType::Float,
                    "Text" => KairoType::Text,
                    "Bool" => KairoType::Bool,
                    _ => KairoType::Generic(generic_param.to_string()),
                };
                KairoType::List(Box::new(inner_type))
            } else {
                KairoType::List(Box::new(KairoType::Generic("T".to_string())))
            }
        } else {
            match type_name.as_str() {
                "Int" => KairoType::Int,
                "Float" => KairoType::Float,
                "Text" => KairoType::Text,
                "Bool" => KairoType::Bool,
                "Map" => KairoType::Map(Box::new(KairoType::Text), Box::new(KairoType::Int)),
                _ => KairoType::Text,
            }
        };
        parameters.push(Parameter { name: "this".to_string(), mutable: false, param_type: this_param_type, default_value: None, variadic: false });

        if !self.check(&TokenType::RightParen) {
            loop {
                let param_name = if let Some(token) = self.current_token() {
                    if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); self.advance(); name }
                    else { return Err(KairoError::syntax("期望参数名".to_string(), token.line, token.column)); }
                } else { 
                    // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
                    let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                        let prev_token = &self.tokens[self.current - 1];
                        (prev_token.line, prev_token.column)
                    } else {
                        (1, 1)
                    };
                    return Err(KairoError::syntax("期望参数名".to_string(), line, column)); 
                };
                let mutable = if self.check(&TokenType::Exclamation) { self.advance(); true } else { false };
                self.consume(TokenType::Colon, "期望 ':'")?;
                let param_type = self.parse_type()?;
                let default_value = if self.check(&TokenType::Assign) { self.advance(); Some(self.parse_expression()?) } else { None };
                let variadic = if self.check(&TokenType::Ellipsis) { self.advance(); true } else { false };
                parameters.push(Parameter { name: param_name, mutable, param_type, default_value, variadic });
                if !self.check(&TokenType::Comma) { break; }
                self.advance();
            }
        }
        self.consume(TokenType::RightParen, "期望 ')'")?;

        let return_type = if self.check(&TokenType::Arrow) { self.advance(); Some(self.parse_type()?) } else { None };
        self.skip_comments_and_newlines();

        if self.check(&TokenType::Assign) {
            self.advance();
            let body_expr = self.parse_expression()?;
            return Ok(Statement { kind: StatementKind::ExtensionFunction { type_name, method_name, parameters, return_type, body: None, body_expr: Some(body_expr) }, line: self.current_token().map(|t| t.line).unwrap_or(1), column: self.current_token().map(|t| t.column).unwrap_or(1) });
        } else {
            self.skip_comments_and_newlines();
            let body = self.parse_block()?;
            return Ok(Statement { kind: StatementKind::ExtensionFunction { type_name, method_name, parameters, return_type, body: Some(body), body_expr: None }, line: self.current_token().map(|t| t.line).unwrap_or(1), column: self.current_token().map(|t| t.column).unwrap_or(1) });
        }
    }

    pub(crate) fn parse_regular_function_body(&mut self, name: String) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "期望 '('")?;
        let mut parameters = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                let param_name = if let Some(token) = self.current_token() {
                    if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); self.advance(); name }
                    else { return Err(KairoError::syntax("期望参数名".to_string(), token.line, token.column)); }
                } else { 
                    // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
                    let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                        let prev_token = &self.tokens[self.current - 1];
                        (prev_token.line, prev_token.column)
                    } else {
                        (1, 1)
                    };
                    return Err(KairoError::syntax("期望参数名".to_string(), line, column)); 
                };
                let mutable = if self.check(&TokenType::Exclamation) { self.advance(); true } else { false };
                self.consume(TokenType::Colon, "期望 ':'")?;
                let param_type = self.parse_type()?;
                let default_value = if self.check(&TokenType::Assign) { self.advance(); Some(self.parse_expression()?) } else { None };
                let variadic = if self.check(&TokenType::Ellipsis) { self.advance(); true } else { false };
                parameters.push(Parameter { name: param_name, mutable, param_type, default_value, variadic });
                if !self.check(&TokenType::Comma) { break; }
                self.advance();
            }
        }
        self.consume(TokenType::RightParen, "期望 ')'")?;
        let return_type = if self.check(&TokenType::Arrow) { self.advance(); Some(self.parse_type()?) } else { None };
        self.skip_comments_and_newlines();
        if self.check(&TokenType::Assign) {
            self.advance();
            let body_expr = self.parse_expression()?;
            return Ok(Statement { kind: StatementKind::FunctionDeclaration { name, parameters, return_type, body: None, body_expr: Some(body_expr) }, line: self.current_token().map(|t| t.line).unwrap_or(1), column: self.current_token().map(|t| t.column).unwrap_or(1) });
        } else {
            self.skip_comments_and_newlines();
            let body = self.parse_block()?;
            return Ok(Statement { kind: StatementKind::FunctionDeclaration { name, parameters, return_type, body: Some(body), body_expr: None }, line: self.current_token().map(|t| t.line).unwrap_or(1), column: self.current_token().map(|t| t.column).unwrap_or(1) });
        }
    }

    pub(crate) fn parse_return_statement(&mut self) -> Result<Statement> {
        let (line, column) = if let Some(token) = self.current_token() { 
            (token.line, token.column) 
        } else { 
            if self.current > 0 && self.current <= self.tokens.len() {
                let prev_token = &self.tokens[self.current - 1];
                (prev_token.line, prev_token.column)
            } else {
                (1, 1)
            }
        };
        self.consume(TokenType::Return, "期望 'return'")?;
        let value = if self.check(&TokenType::Newline) || self.is_at_end() { None } else { Some(self.parse_expression()?) };
        Ok(Statement { kind: StatementKind::Return { value }, line, column })
    }
}
