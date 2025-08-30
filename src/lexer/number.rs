// SPDX-License-Identifier: Apache-2.0

//! 数字字面量解析（支持整数与浮点）

use super::token::TokenType;

impl super::Lexer {
    pub(super) fn read_number(&mut self) -> TokenType {
        let mut number = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek().map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            TokenType::FloatLiteral(number.parse().unwrap())
        } else {
            TokenType::IntLiteral(number.parse().unwrap())
        }
    }
}
