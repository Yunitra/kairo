// SPDX-License-Identifier: Apache-2.0

//! 语句解析

use crate::ast::{Statement, Block, MatchArm, Program};
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
                TokenType::Return => { return self.parse_return_statement(); }
                TokenType::If => { return self.parse_if_statement(); }
                TokenType::While => { return self.parse_while_statement(); }
                TokenType::For => { return self.parse_for_statement(); }
                TokenType::Match => { return self.parse_match_statement(); }
                TokenType::Break => { return self.parse_break_statement(); }
                TokenType::Continue => { self.advance(); return Ok(Statement::Continue); }
                TokenType::LeftBrace => { return self.parse_block_statement(); }
                TokenType::Identifier(_) => {
                    if let Some(next_token) = self.tokens.get(self.current + 1) {
                        match &next_token.token_type {
                            TokenType::Assign | TokenType::Exclamation | TokenType::Colon => { return self.parse_variable_declaration(); }
                            TokenType::PlusAssign | TokenType::MinusAssign | TokenType::MultiplyAssign | TokenType::DivideAssign => { return self.parse_assignment_statement(); }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        let expr = self.parse_expression()?;
        Ok(Statement::Expression(expr))
    }

    pub(crate) fn parse_const_declaration(&mut self) -> Result<Statement> {
        self.consume(TokenType::Const, "期望 'const'")?;
        let (name, _line, _column) = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); let line = token.line; let column = token.column; self.advance(); (name, line, column) }
            else { return Err(KairoError::syntax("期望常量名".to_string(), token.line, token.column)); }
        } else { return Err(KairoError::syntax("期望常量名".to_string(), 1, 1)); };
        self.consume(TokenType::Assign, "期望 '='")?;
        let value = self.parse_expression()?;
        Ok(Statement::VariableDeclaration { name, mutable: false, explicit_type: None, value, is_const: true })
    }

    pub(crate) fn parse_variable_declaration(&mut self) -> Result<Statement> {
        let (name, _line, _column) = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); let line = token.line; let column = token.column; self.advance(); (name, line, column) }
            else { return Err(KairoError::syntax("期望变量名".to_string(), token.line, token.column)); }
        } else { return Err(KairoError::syntax("期望变量名".to_string(), 1, 1)); };
        let mut mutable = false;
        let mut explicit_type = None;
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Exclamation) { mutable = true; self.advance(); }
        }
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Colon) { self.advance(); explicit_type = Some(self.parse_type()?); }
        }
        self.consume(TokenType::Assign, "期望 '='")?;
        let value = self.parse_expression()?;
        Ok(Statement::VariableDeclaration { name, mutable, explicit_type, value, is_const: false })
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

    pub(crate) fn parse_block_statement(&mut self) -> Result<Statement> { let block = self.parse_block()?; Ok(Statement::Block(block)) }

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
        Ok(Statement::If { condition, then_branch, else_ifs, else_branch })
    }

    pub(crate) fn parse_while_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::While, "期望 'while'")?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement::While { condition, body })
    }

    pub(crate) fn parse_for_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::For, "期望 'for'")?;
        let variable = if let Some(token) = self.current_token() {
            if let TokenType::Identifier(name) = &token.token_type { let name = name.clone(); self.advance(); name }
            else { return Err(KairoError::syntax("期望变量名".to_string(), token.line, token.column)); }
        } else { return Err(KairoError::syntax("期望变量名".to_string(), 1, 1)); };
        let mut value_variable = None;
        if self.check(&TokenType::Comma) { self.advance(); if let Some(token) = self.current_token() { if let TokenType::Identifier(name) = &token.token_type { value_variable = Some(name.clone()); self.advance(); } else { return Err(KairoError::syntax("期望变量名".to_string(), token.line, token.column)); } } }
        self.consume(TokenType::In, "期望 'in'")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement::For { variable, value_variable, iterable, body })
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
        Ok(Statement::Match { value, arms })
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
        Ok(Statement::Break { levels })
    }
}
