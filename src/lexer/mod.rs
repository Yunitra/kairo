// SPDX-License-Identifier: Apache-2.0

//! 词法分析器模块入口
//!
//! - `token.rs` 定义了 `TokenType` 与 `Token`。
//! - 子模块按职责拆分：
//!   - `reader.rs`：字符读取与游标控制
//!   - `string.rs`：字符串与文档字符串
//!   - `number.rs`：数字字面量
//!   - `identifier.rs`：标识符与关键字
//!   - `comment.rs`：注释
//!   - `scanner.rs`：`next_token`/`tokenize` 主流程
//! - 对外导出 `Token/TokenType/Lexer`。

mod token;
mod reader;
mod string;
mod number;
mod identifier;
mod comment;
mod scanner;

pub use token::{Token, TokenType};

pub struct Lexer {
    pub(super) input: Vec<char>,
    pub(super) position: usize,
    pub(super) current_char: Option<char>,
    pub(super) line: usize,
    pub(super) column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        
        Self {
            input: chars,
            position: 0,
            current_char,
            line: 1,
            column: 1,
        }
    }
}
