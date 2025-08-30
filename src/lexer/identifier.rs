// SPDX-License-Identifier: Apache-2.0

//! 标识符与关键字解析

use super::token::TokenType;

impl super::Lexer {
    pub(super) fn read_identifier(&mut self) -> String {
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        identifier
    }

    pub(super) fn keyword_or_identifier(&mut self) -> TokenType {
        let identifier = self.read_identifier();
        match identifier.as_str() {
            "const" => TokenType::Const,
            "true" => TokenType::BoolLiteral(true),
            "false" => TokenType::BoolLiteral(false),
            "null" => TokenType::Null,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "match" => TokenType::Match,
            "case" => TokenType::Case,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "fun" => TokenType::Fun,
            "return" => TokenType::Return,
            "Void" => TokenType::Void,
            // 类型名也允许作为标识符出现（交给 parser 处理具体含义）
            "List" => TokenType::Identifier(identifier),
            "Map" => TokenType::Identifier(identifier),
            _ => TokenType::Identifier(identifier),
        }
    }
}
