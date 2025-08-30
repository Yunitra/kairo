// SPDX-License-Identifier: Apache-2.0

//! 作用域与符号表：变量与函数定义的存取

use std::collections::HashMap;

use crate::error::{KairoError, Result};
use crate::types::KairoValue;

use super::function::FunctionDef;

#[derive(Debug)]
pub struct Scope {
    pub(super) variables: HashMap<String, (KairoValue, bool)>,
    pub(super) functions: HashMap<String, FunctionDef>,
}

impl Scope {
    pub(super) fn new() -> Self {
        Self { variables: HashMap::new(), functions: HashMap::new() }
    }
}

impl super::Interpreter {
    pub(super) fn push_scope(&mut self) { self.scopes.push(Scope::new()); }
    pub(super) fn pop_scope(&mut self) { if self.scopes.len() > 1 { self.scopes.pop(); } }

    pub(super) fn set_variable(&mut self, name: String, value: KairoValue, mutable: bool, line: usize, column: usize) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() { scope.variables.insert(name, (value, mutable)); Ok(()) } else { Err(KairoError::runtime("没有可用的作用域".to_string(), line, column)) }
    }

    pub(super) fn get_variable(&self, name: &str) -> Option<(KairoValue, bool)> {
        for scope in self.scopes.iter().rev() { if let Some((value, mutable)) = scope.variables.get(name) { return Some((value.clone(), *mutable)); } }
        None
    }

    pub(super) fn update_variable(&mut self, name: &str, value: KairoValue, line: usize, column: usize) -> Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some((stored_value, mutable)) = scope.variables.get_mut(name) {
                if !*mutable { return Err(KairoError::runtime(format!("尝试修改不可变变量: {}", name), line, column)); }
                *stored_value = value; return Ok(());
            }
        }
        Err(KairoError::runtime(format!("未定义的变量: {}", name), line, column))
    }

    pub(super) fn set_function(&mut self, name: String, function: FunctionDef, line: usize, column: usize) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() { scope.functions.insert(name, function); Ok(()) } else { Err(KairoError::runtime("没有可用的作用域".to_string(), line, column)) }
    }

    pub(super) fn get_function(&self, name: &str) -> Option<FunctionDef> {
        for scope in self.scopes.iter().rev() { if let Some(function) = scope.functions.get(name) { return Some(function.clone()); } }
        None
    }
}
