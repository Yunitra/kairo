// SPDX-License-Identifier: Apache-2.0

//! 注释解析：单行、多行与文档注释

use crate::error::{KairoError, Result};

impl super::Lexer {
    pub(super) fn read_single_comment(&mut self) -> String {
        let mut comment = String::new();
        self.advance(); // 跳过 #

        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            comment.push(ch);
            self.advance();
        }

        comment.trim().to_string()
    }

    pub(super) fn read_multi_comment(&mut self) -> Result<String> {
        let mut comment = String::new();
        self.advance(); // 跳过第一个 -
        self.advance(); // 跳过第二个 -
        self.advance(); // 跳过第三个 -

        loop {
            match self.current_char {
                Some('-') if self.peek() == Some('-') => {
                    // 检查是否是结束的 ---
                    let pos = self.position;
                    self.advance();
                    self.advance();
                    if self.current_char == Some('-') {
                        self.advance();
                        break;
                    } else {
                        // 恢复位置并继续
                        self.position = pos;
                        self.current_char = self.input.get(self.position).copied();
                        comment.push('-');
                        self.advance();
                    }
                }
                Some(ch) => {
                    comment.push(ch);
                    self.advance();
                }
                None => {
                    return Err(KairoError::lexical(
                        "未闭合的多行注释".to_string(),
                        self.line,
                        self.column,
                    ));
                }
            }
        }

        Ok(comment.trim().to_string())
    }
}
