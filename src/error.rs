// SPDX-License-Identifier: Apache-2.0

use colored::*;
use std::fmt;

#[derive(Debug, Clone)]
pub struct KairoError {
    pub kind: ErrorKind,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub source_line: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    LexicalError,
    SyntaxError,
    TypeError,
    RuntimeError,
    IoError,
    /// An internal signal to propagate control flow changes, not a user-facing error.
    ControlFlowSignal(super::interpreter::ControlFlow),
}

impl KairoError {
    pub fn new(kind: ErrorKind, message: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            message,
            line,
            column,
            source_line: None,
        }
    }

    pub fn with_source_line(mut self, source_line: String) -> Self {
        self.source_line = Some(source_line);
        self
    }

    pub fn lexical(message: String, line: usize, column: usize) -> Self {
        Self::new(ErrorKind::LexicalError, message, line, column)
    }

    pub fn syntax(message: String, line: usize, column: usize) -> Self {
        Self::new(ErrorKind::SyntaxError, message, line, column)
    }

    pub fn type_error(message: String, line: usize, column: usize) -> Self {
        Self::new(ErrorKind::TypeError, message, line, column)
    }

    pub fn runtime(message: String, line: usize, column: usize) -> Self {
        Self::new(ErrorKind::RuntimeError, message, line, column)
    }

    pub fn control_flow_signal(control_flow: super::interpreter::ControlFlow, message: &str) -> Self {
        Self::new(ErrorKind::ControlFlowSignal(control_flow), message.to_string(), 0, 0)
    }
}

impl fmt::Display for KairoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_type = match self.kind {
            ErrorKind::LexicalError => "词法错误".red(),
            ErrorKind::SyntaxError => "语法错误".red(),
            ErrorKind::TypeError => "类型错误".red(),
            ErrorKind::RuntimeError => "运行时错误".red(),
            ErrorKind::IoError => "IO错误".red(),
            
            // 此变体仅供内部使用，不应展示给用户
            ErrorKind::ControlFlowSignal(_) => "内部控制流错误".bright_black(),
        };

        // 对于内部控制流错误，不打印源代码行
        if matches!(self.kind, ErrorKind::ControlFlowSignal(_)) {
            return write!(f, "{}: {}", error_type, self.message);
        }

        writeln!(f, "{}: {}", error_type, self.message)?;
        writeln!(f, "  --> 第{}行，第{}列", self.line, self.column)?;
        
        if let Some(ref source) = self.source_line {
            writeln!(f, "     |")?;
            writeln!(f, " {:3} | {}", self.line, source)?;
            writeln!(f, "     | {}^", " ".repeat(self.column.saturating_sub(1)))?;
        }

        Ok(())
    }
}

impl std::error::Error for KairoError {}

pub type Result<T> = std::result::Result<T, KairoError>;
