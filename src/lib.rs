// SPDX-License-Identifier: Apache-2.0

//! Kairo 对外库入口
//! 该文件聚合导出各子模块的公共 API。

pub mod lexer;       // 目录模块：包含 token/reader/.../scanner
pub mod parser;      // 目录模块：拆分 expressions/statements/... 等
pub mod ast;         // 目录模块入口，导出表达式/语句/通用结构
pub mod interpreter; // 目录模块：解释器主逻辑与子模块
pub mod types;       // 目录模块入口，导出 type/value
pub mod error;

// 公共导出：保持对外 API 稳定
pub use error::*;
pub use types::*;       // 导出 KairoType/KairoValue
pub use ast::*;         // 导出 AST 结构
pub use lexer::*;       // 导出 Lexer/Token/TokenType
pub use parser::*;      // 导出 Parser
pub use interpreter::*;
