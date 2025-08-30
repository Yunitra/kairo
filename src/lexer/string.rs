// SPDX-License-Identifier: Apache-2.0

//! 字符串与文档字符串解析

use crate::error::{KairoError, Result};

impl super::Lexer {
    pub(super) fn read_string(&mut self, quote: char) -> Result<String> {
        let mut string = String::new();
        self.advance(); // 跳过开始的引号

        while let Some(ch) = self.current_char {
            if ch == quote {
                self.advance(); // 跳过结束的引号
                return Ok(string);
            } else if ch == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => string.push('\n'),
                    Some('t') => string.push('\t'),
                    Some('r') => string.push('\r'),
                    Some('\\') => string.push('\\'),
                    Some('"') => string.push('"'),
                    Some('\'') => string.push('\''),
                    Some(c) => {
                        return Err(KairoError::lexical(
                            format!("无效的转义字符: \\{}", c),
                            self.line,
                            self.column,
                        ));
                    }
                    None => {
                        return Err(KairoError::lexical(
                            "字符串中的转义字符不完整".to_string(),
                            self.line,
                            self.column,
                        ));
                    }
                }
                self.advance();
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Err(KairoError::lexical(
            "未闭合的字符串".to_string(),
            self.line,
            self.column,
        ))
    }

    pub(super) fn read_doc_comment(&mut self) -> String {
        let mut comment = String::new();
        self.advance(); // 跳过第一个 *
        self.advance(); // 跳过第二个 *

        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            comment.push(ch);
            self.advance();
        }

        comment.trim().to_string()
    }
}
