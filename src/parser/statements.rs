// SPDX-License-Identifier: Apache-2.0

//! 语句解析

use crate::ast::{Statement, StatementKind, Block, MatchArm, Program, ErrorField};
use crate::error::{KairoError, Result};
use crate::lexer::TokenType;

use super::Parser;

impl Parser {
    pub(crate) fn statements_parse_program(&mut self) -> Result<Program> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            self.skip_comments_and_newlines();
            if self.is_at_end() { break; }
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_comments_and_newlines();
        }
        Ok(Program { statements })
    }

    pub(crate) fn parse_statement(&mut self) -> Result<Statement> {
        if let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::Const => { return self.parse_const_declaration(); }
                TokenType::Fun => { return self.parse_function_declaration(); }
                TokenType::Err => { return self.parse_error_definition(); }
                TokenType::Fail => { return self.parse_fail_statement(); }
                TokenType::Return => { return self.parse_return_statement(); }
                TokenType::If => { return self.parse_if_statement(); }
                TokenType::While => { return self.parse_while_statement(); }
                TokenType::For => { return self.parse_for_statement(); }
                TokenType::Match => { return self.parse_match_statement(); }
                TokenType::Break => { return self.parse_break_statement(); }
                TokenType::Continue => { let t = token.clone(); self.advance(); return Ok(Statement { kind: StatementKind::Continue, line: t.line, column: t.column }); }
                TokenType::LeftBrace => { return self.parse_block_statement(); }
                TokenType::Identifier(_) => {
                    if let Some(next_token) = self.tokens.get(self.current + 1) {
                        match &next_token.token_type {
                            TokenType::Assign | TokenType::Exclamation | TokenType::Colon | TokenType::Question => { return self.parse_variable_declaration(); }
                            TokenType::PlusAssign | TokenType::MinusAssign | TokenType::MultiplyAssign | TokenType::DivideAssign => { return self.parse_assignment_statement(); }
                            _ => {}
                        }
                    }
                }
                TokenType::Dollar => { return self.parse_variable_declaration(); }
                _ => {}
            }
        }
        let expr = self.parse_expression()?;
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((1,1));
        Ok(Statement { kind: StatementKind::Expression(expr), line, column })
    }

    pub(crate) fn parse_const_declaration(&mut self) -> Result<Statement> {
        self.consume(TokenType::Const, "期望 'const'")?;
        let (name, _line, _column) = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); let line = token.line; let column = token.column; self.advance(); (name, line, column) }
            else { return Err(KairoError::syntax("期望常量名".to_string(), token.line, token.column)); }
        } else { 
            // 如果没有当前token，尝试从上一个token获取位置，或者使用默认位置
            let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
                let prev_token = &self.tokens[self.current - 1];
                (prev_token.line, prev_token.column)
            } else {
                (1, 1)
            };
            return Err(KairoError::syntax("期望常量名".to_string(), line, column)); 
        };
        self.consume(TokenType::Assign, "期望 '='")?;
        let value = self.parse_expression()?;
        // 使用常量声明的开始位置
        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
            let prev_token = &self.tokens[self.current - 1];
            (prev_token.line, prev_token.column)
        } else {
            (1, 1)
        };
        Ok(Statement { kind: StatementKind::VariableDeclaration { name, mutable: false, explicit_type: None, value, is_const: true }, line, column })
    }

    pub(crate) fn parse_variable_declaration(&mut self) -> Result<Statement> {
        let mut mutable = false;
        
        // 检查是否以 $ 开头（可变变量）
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Dollar) {
                mutable = true;
                self.advance();
            }
        }
        
        let (name, _line, _column) = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type { 
                let name = name.clone(); 
                let line = token.line; 
                let column = token.column; 
                self.advance(); 
                (name, line, column) 
            }
            else { return Err(KairoError::syntax("期望变量名".to_string(), token.line, token.column)); }
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
        
        let mut explicit_type = None;
        
        // 检查是否有 ? 表示可空类型
        let mut is_nullable = false;
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Question) {
                is_nullable = true;
                self.advance();
            }
        }
        
        // 检查老式 ! 语法（向后兼容）
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Exclamation) { 
                if !mutable {  // 如果没有用 $ 前缀，那么 ! 表示可变
                    mutable = true; 
                }
                self.advance(); 
            }
        }
        
        // 解析显式类型注解
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Colon) { 
                self.advance(); 
                let mut parsed_type = self.parse_type()?;
                
                // 如果有 ? 标记，包装为可空类型
                if is_nullable {
                    parsed_type = crate::types::KairoType::Nullable(Box::new(parsed_type));
                }
                
                explicit_type = Some(parsed_type);
            }
        }
        
        self.consume(TokenType::Assign, "期望 '='")?;
        let value = self.parse_expression()?;
        // 使用变量声明的开始位置
        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
            let prev_token = &self.tokens[self.current - 1];
            (prev_token.line, prev_token.column)
        } else {
            (1, 1)
        };
        Ok(Statement { kind: StatementKind::VariableDeclaration { name, mutable, explicit_type, value, is_const: false }, line, column })
    }

    pub(crate) fn parse_block(&mut self) -> Result<Block> {
        self.consume(TokenType::LeftBrace, "期望 '{'")?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            self.skip_comments_and_newlines();
            if self.check(&TokenType::RightBrace) { break; }
            statements.push(self.parse_statement()?);
            self.skip_comments_and_newlines();
        }
        self.consume(TokenType::RightBrace, "期望 '}'")?;
        Ok(Block { statements })
    }

    pub(crate) fn parse_block_statement(&mut self) -> Result<Statement> { 
        let block = self.parse_block()?; 
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or_else(|| {
            if self.current > 0 && self.current <= self.tokens.len() {
                let prev_token = &self.tokens[self.current - 1];
                (prev_token.line, prev_token.column)
            } else {
                (1, 1)
            }
        }); 
        Ok(Statement { kind: StatementKind::Block(block), line, column }) 
    }

    pub(crate) fn parse_if_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::If, "期望 'if'")?;
        let has_paren = self.check(&TokenType::LeftParen);
        if has_paren { self.advance(); }
        let condition = self.parse_expression()?;
        if has_paren { self.consume(TokenType::RightParen, "期望 ')'")?; }
        let then_branch = self.parse_block()?;
        let mut else_ifs = Vec::new();
        let mut else_branch = None;
        while self.check(&TokenType::Else) {
            self.advance();
            if self.check(&TokenType::If) {
                self.advance();
                let has_paren = self.check(&TokenType::LeftParen);
                if has_paren { self.advance(); }
                let else_if_condition = self.parse_expression()?;
                if has_paren { self.consume(TokenType::RightParen, "期望 ')'")?; }
                let else_if_body = self.parse_block()?;
                else_ifs.push((else_if_condition, else_if_body));
            } else { else_branch = Some(self.parse_block()?); break; }
        }
        // 使用条件表达式的行号作为if语句的位置
        let (line, column) = (condition.line, condition.column);
        Ok(Statement { kind: StatementKind::If { condition, then_branch, else_ifs, else_branch }, line, column })
    }

    pub(crate) fn parse_while_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::While, "期望 'while'")?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        // 使用条件表达式的行号作为while语句的位置
        let (line, column) = (condition.line, condition.column);
        Ok(Statement { kind: StatementKind::While { condition, body }, line, column })
    }

    pub(crate) fn parse_for_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::For, "期望 'for'")?;
        let variable = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); self.advance(); name }
            else { return Err(KairoError::syntax("期望变量名".to_string(), token.line, token.column)); }
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
        let mut value_variable = None;
        if self.check(&TokenType::Comma) { self.advance(); if let Some(token) = self.current_token() { if let TokenType::Identifier(name) = &token.token_type { value_variable = Some(name.clone()); self.advance(); } else { return Err(KairoError::syntax("期望变量名".to_string(), token.line, token.column)); } } }
        self.consume(TokenType::In, "期望 'in'")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        // 使用for语句开始的位置
        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
            let prev_token = &self.tokens[self.current - 1];
            (prev_token.line, prev_token.column)
        } else {
            (1, 1)
        };
        Ok(Statement { kind: StatementKind::For { variable, value_variable, iterable, body }, line, column })
    }

    pub(crate) fn parse_match_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::Match, "期望 'match'")?;
        let value = self.parse_expression()?;
        self.consume(TokenType::LeftBrace, "期望 '{'")?;
        let mut arms = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            self.skip_comments_and_newlines();
            if self.check(&TokenType::RightBrace) { break; }
            let arm = self.parse_match_arm()?;
            arms.push(arm);
            self.skip_comments_and_newlines();
        }
        self.consume(TokenType::RightBrace, "期望 '}'")?;
        // 使用match语句开始的位置
        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
            let prev_token = &self.tokens[self.current - 1];
            (prev_token.line, prev_token.column)
        } else {
            (1, 1)
        };
        Ok(Statement { kind: StatementKind::Match { value, arms }, line, column })
    }

    pub(crate) fn parse_match_arm(&mut self) -> Result<MatchArm> {
        let pattern = self.parse_pattern()?;
        let guard = if self.check(&TokenType::If) { self.advance(); Some(self.parse_expression()?) } else { None };
        self.consume(TokenType::Arrow, "期望 '->'")?;
        let body = self.parse_block()?;
        Ok(MatchArm { pattern, guard, body })
    }

    pub(crate) fn parse_break_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::Break, "期望 'break'")?;
        let levels = if let Some(token) = self.current_token() { if let TokenType::IntLiteral(n) = &token.token_type { let levels = *n as usize; self.advance(); Some(levels) } else { None } } else { None };
        // 使用break语句的位置
        let (line, column) = if self.current > 0 && self.current <= self.tokens.len() {
            let prev_token = &self.tokens[self.current - 1];
            (prev_token.line, prev_token.column)
        } else {
            (1, 1)
        };
        Ok(Statement { kind: StatementKind::Break { levels }, line, column })
    }

    /// 解析错误定义语句
    /// 语法：err ErrorName 或 err ErrorName { field: Type } 或 err GroupName = Error1, Error2
    pub(crate) fn parse_error_definition(&mut self) -> Result<Statement> {
        let start_token = self.consume(TokenType::Err, "期望 'err'")?;
        
        // 错误名称
        let name = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(KairoError::syntax(
                    "期望错误名称".to_string(),
                    token.line,
                    token.column,
                ));
            }
        } else {
            return Err(KairoError::syntax(
                "期望错误名称".to_string(),
                start_token.line,
                start_token.column,
            ));
        };

        // 检查是否是错误组定义
        if self.check(&TokenType::Assign) {
            self.advance(); // consume '='
            let mut errors = Vec::new();
            
            // 解析错误组成员
            loop {
                if let Some(token) = self.current_token() {
                    if let TokenType::Identifier(error_name) = &token.token_type {
                        errors.push(error_name.clone());
                        self.advance();
                        
                        if self.check(&TokenType::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    } else {
                        return Err(KairoError::syntax(
                            "期望错误名称".to_string(),
                            token.line,
                            token.column,
                        ));
                    }
                } else {
                    break;
                }
            }
            
            return Ok(Statement {
                kind: StatementKind::ErrorDefinition {
                    name,
                    fields: None,
                    error_group: Some(errors),
                },
                line: start_token.line,
                column: start_token.column,
            });
        }

        // 检查是否有字段定义
        let fields = if self.check(&TokenType::LeftBrace) {
            self.advance(); // consume '{'
            let mut fields = Vec::new();
            
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                // 字段名
                let field_name = if let Some(token) = self.current_token() {
                    if let TokenType::Identifier(name) = &token.token_type {
                        let name = name.clone();
                        self.advance();
                        name
                    } else {
                        return Err(KairoError::syntax(
                            "期望字段名称".to_string(),
                            token.line,
                            token.column,
                        ));
                    }
                } else {
                    return Err(KairoError::syntax(
                        "期望字段名称".to_string(),
                        start_token.line,
                        start_token.column,
                    ));
                };
                
                self.consume(TokenType::Colon, "期望 ':'")?;
                
                // 字段类型
                let field_type = self.parse_type()?;
                
                fields.push(ErrorField {
                    name: field_name,
                    field_type,
                });
                
                if self.check(&TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            
            self.consume(TokenType::RightBrace, "期望 '}'")?;
            Some(fields)
        } else {
            None
        };

        Ok(Statement {
            kind: StatementKind::ErrorDefinition {
                name,
                fields,
                error_group: None,
            },
            line: start_token.line,
            column: start_token.column,
        })
    }

    /// 解析 fail 语句
    /// 语法：fail ErrorName 或 fail ErrorName(field: value, ...)
    pub(crate) fn parse_fail_statement(&mut self) -> Result<Statement> {
        let start_token = self.consume(TokenType::Fail, "期望 'fail'")?;
        
        // 错误名称
        let error_name = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(KairoError::syntax(
                    "期望错误名称".to_string(),
                    token.line,
                    token.column,
                ));
            }
        } else {
            return Err(KairoError::syntax(
                "期望错误名称".to_string(),
                start_token.line,
                start_token.column,
            ));
        };

        // 检查是否有错误数据
        let data = if self.check(&TokenType::LeftParen) {
            self.advance(); // consume '('
            let mut data = Vec::new();
            
            while !self.check(&TokenType::RightParen) && !self.is_at_end() {
                // 字段名
                let field_name = if let Some(token) = self.current_token() {
                    if let TokenType::Identifier(name) = &token.token_type {
                        let name = name.clone();
                        self.advance();
                        name
                    } else {
                        return Err(KairoError::syntax(
                            "期望字段名称".to_string(),
                            token.line,
                            token.column,
                        ));
                    }
                } else {
                    return Err(KairoError::syntax(
                        "期望字段名称".to_string(),
                        start_token.line,
                        start_token.column,
                    ));
                };
                
                self.consume(TokenType::Colon, "期望 ':'")?;
                
                // 字段值表达式
                let field_value = self.parse_expression()?;
                
                data.push((field_name, field_value));
                
                if self.check(&TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            
            self.consume(TokenType::RightParen, "期望 ')'")?;
            Some(data)
        } else {
            None
        };

        Ok(Statement {
            kind: StatementKind::Fail {
                error_name,
                data,
            },
            line: start_token.line,
            column: start_token.column,
        })
    }
}
