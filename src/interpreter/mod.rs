// SPDX-License-Identifier: Apache-2.0

//! 解释器模块入口
//!
//! - `control_flow.rs`：控制流枚举
//! - `scope.rs`：作用域与变量/函数存取
//! - `function.rs`：函数定义与调用
//! - 本文件：`Interpreter` 本体与执行/求值逻辑

mod control_flow;
mod scope;
mod function;
mod pattern;
mod exec;
mod eval;

use crate::ast::Program;
use crate::error::Result;

pub use control_flow::ControlFlow;
pub use function::{FunctionDef, FunctionBodyType};
pub use scope::Scope;

pub struct Interpreter {
    pub(super) scopes: Vec<Scope>,
    pub(super) control_flow: ControlFlow,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            scopes: Vec::new(),
            control_flow: ControlFlow::None,
        };
        interpreter.push_scope(); // 全局作用域
        interpreter
    }

    pub fn interpret(&mut self, program: Program) -> Result<()> {
        for statement in program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }
}
