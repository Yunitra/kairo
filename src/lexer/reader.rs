// SPDX-License-Identifier: Apache-2.0

//! 字符读取与基础游标控制

impl super::Lexer {
    /// 前进一个字符，同时维护行列号
    pub(super) fn advance(&mut self) {
        if self.current_char == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    /// 预读下一个字符但不消耗
    pub(super) fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    /// 跳过空白（不含换行）
    pub(super) fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
}
