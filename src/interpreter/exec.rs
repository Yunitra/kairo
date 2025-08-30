// SPDX-License-Identifier: Apache-2.0

//! 语句执行：`execute_statement` 与 `execute_block`

use crate::ast::{Statement, StatementKind, Expression, Block};
use crate::error::{KairoError, Result};

use crate::types::KairoValue;
use super::{Interpreter, ControlFlow, FunctionBodyType, FunctionDef};

impl Interpreter {
    pub(super) fn execute_statement(&mut self, statement: Statement) -> Result<()> {
        match statement.kind {
            StatementKind::VariableDeclaration { name, mutable, explicit_type, value, is_const: _ } => {
                let computed_value = self.evaluate_expression(value)?;
                if let Some(expected_type) = explicit_type {
                    let actual_type = computed_value.get_type();
                    if !self.types_match(&expected_type, &actual_type) {
                        return Err(KairoError::type_error(
                            format!("类型不匹配: 期望 {}, 但得到 {}", expected_type, actual_type), statement.line, statement.column,
                        ));
                    }
                }
                self.set_variable(name, computed_value, mutable, statement.line, statement.column)?;
            }
            StatementKind::FunctionDeclaration { name, parameters, return_type, body, body_expr, raises } => {
                let params: Vec<(String, crate::types::KairoType, bool, Option<Expression>, bool)> = parameters
                    .into_iter()
                    .map(|p| (p.name, p.param_type, p.mutable, p.default_value, p.variadic))
                    .collect();
                let function_body = if let Some(expr) = body_expr { FunctionBodyType::Expression(expr) } else if let Some(block) = body { FunctionBodyType::Block(block) } else { return Err(KairoError::runtime("函数必须有函数体".to_string(), statement.line, statement.column)); };
                let function_def = FunctionDef { name: name.clone(), parameters: params, return_type, body: function_body, raises, line: statement.line, column: statement.column };
                self.set_function(name, function_def, statement.line, statement.column)?;
            }
            StatementKind::ExtensionFunction { type_name, method_name, parameters, return_type, body, body_expr, raises } => {
                let params: Vec<(String, crate::types::KairoType, bool, Option<Expression>, bool)> = parameters
                    .into_iter()
                    .map(|p| (p.name, p.param_type, p.mutable, p.default_value, p.variadic))
                    .collect();
                let function_body = if let Some(expr) = body_expr { FunctionBodyType::Expression(expr) } else if let Some(block) = body { FunctionBodyType::Block(block) } else { return Err(KairoError::runtime("扩展函数必须有函数体".to_string(), statement.line, statement.column)); };
                let full_name = format!("{}.{}", type_name, method_name);
                let function_def = FunctionDef { name: full_name.clone(), parameters: params, return_type, body: function_body, raises, line: statement.line, column: statement.column };
                self.set_function(full_name, function_def, statement.line, statement.column)?;
            }
            StatementKind::Return { value } => {
                let return_value = if let Some(expr) = value { self.evaluate_expression(expr)? } else { KairoValue::Unit };
                self.control_flow = ControlFlow::Return(return_value, statement.line, statement.column);
            }
            StatementKind::Expression(expr) => { let _ = self.evaluate_expression(expr)?; }
            StatementKind::If { condition, then_branch, else_ifs, else_branch } => {
                let condition_value = self.evaluate_expression(condition)?;
                if condition_value.is_truthy() { self.execute_block(then_branch)?; } else {
                    let mut executed = false;
                    for (else_if_condition, else_if_body) in else_ifs {
                        let else_if_value = self.evaluate_expression(else_if_condition)?;
                        if else_if_value.is_truthy() { self.execute_block(else_if_body)?; executed = true; break; }
                    }
                    if !executed { if let Some(else_body) = else_branch { self.execute_block(else_body)?; } }
                }
            }
            StatementKind::While { condition, body } => {
                loop {
                    let condition_value = self.evaluate_expression(condition.clone())?;
                    if !condition_value.is_truthy() { break; }
                    self.execute_block(body.clone())?;
                    match &self.control_flow {
                        ControlFlow::Break(levels) => { let levels = *levels; self.control_flow = ControlFlow::None; if levels > 1 { self.control_flow = ControlFlow::Break(levels - 1); } break; }
                        ControlFlow::Continue => { self.control_flow = ControlFlow::None; continue; }
                        ControlFlow::Return(_, _, _) => { break; }
                        ControlFlow::Error(_, _, _) => { break; }
                        ControlFlow::ThrownError { .. } => { break; }
                        ControlFlow::None => {}
                    }
                }
            }
            StatementKind::For { variable, value_variable, iterable, body } => {
                let iterable_value = self.evaluate_expression(iterable)?;
                match iterable_value {
                    KairoValue::List(items) => {
                        for item in items {
                            self.push_scope();
                            self.set_variable(variable.clone(), item, false, statement.line, statement.column)?;
                            self.execute_block(body.clone())?;
                            let should_break = match &self.control_flow {
                                ControlFlow::Break(levels) => { let levels = *levels; self.control_flow = ControlFlow::None; if levels > 1 { self.control_flow = ControlFlow::Break(levels - 1); } true }
                                ControlFlow::Continue => { self.control_flow = ControlFlow::None; false }
                                ControlFlow::Return(_, _, _) => { true }
                                ControlFlow::Error(_, _, _) => { true }
                                ControlFlow::ThrownError { .. } => { true }
                                ControlFlow::None => false,
                            };
                            self.pop_scope();
                            if should_break { break; }
                        }
                    }
                    KairoValue::Map(map) => {
                        for (key, value) in map {
                            self.push_scope();
                            self.set_variable(variable.clone(), KairoValue::Text(key), false, statement.line, statement.column)?;
                            if let Some(value_var) = &value_variable { self.set_variable(value_var.clone(), value, false, statement.line, statement.column)?; }
                            self.execute_block(body.clone())?;
                            let should_break = match &self.control_flow {
                                ControlFlow::Break(levels) => { let levels = *levels; self.control_flow = ControlFlow::None; if levels > 1 { self.control_flow = ControlFlow::Break(levels - 1); } true }
                                ControlFlow::Continue => { self.control_flow = ControlFlow::None; false }
                                ControlFlow::Return(_, _, _) => { true }
                                ControlFlow::Error(_, _, _) => { true }
                                ControlFlow::ThrownError { .. } => { true }
                                ControlFlow::None => false,
                            };
                            self.pop_scope();
                            if should_break { break; }
                        }
                    }
                    _ => { return Err(KairoError::type_error("for 循环只能遍历 List 或 Map".to_string(), statement.line, statement.column)); }
                }
            }
            StatementKind::Match { value, arms } => {
                let match_value = self.evaluate_expression(value)?;
                for arm in arms {
                    if self.pattern_matches(&arm.pattern, &match_value)? {
                        self.push_scope();
                        self.bind_pattern_variables(&arm.pattern, &match_value, statement.line, statement.column)?;
                        if let Some(guard) = arm.guard { let guard_value = self.evaluate_expression(guard)?; if !guard_value.is_truthy() { self.pop_scope(); continue; } }
                        self.execute_block(arm.body)?;
                        self.pop_scope();
                        break;
                    }
                }
            }
            StatementKind::Break { levels } => { self.control_flow = ControlFlow::Break(levels.unwrap_or(1)); }
            StatementKind::Continue => { self.control_flow = ControlFlow::Continue; }
            StatementKind::Block(block) => { 
                self.push_scope(); 
                self.execute_block(block)?; 
                self.pop_scope(); 
            }
            StatementKind::ErrorDefinition { name, fields, error_group } => {
                // 错误定义语句 - 在全局作用域注册错误类型
                if let Some(errors) = error_group {
                    // 错误组定义
                    self.error_handling.register_error_group(name, errors, statement.line, statement.column)?;
                } else {
                    // 单个错误定义
                    self.error_handling.register_error_type(name, fields, statement.line, statement.column)?;
                }
            }
            StatementKind::Fail { error_name, data } => {
                // 抛出错误
                let error_data = if let Some(data_pairs) = data {
                    let mut error_map = std::collections::HashMap::new();
                    for (field_name, field_expr) in data_pairs {
                        let field_value = self.evaluate_expression(field_expr)?;
                        error_map.insert(field_name, field_value);
                    }
                    Some(error_map)
                } else {
                    None
                };
                
                // 设置控制流为抛出错误状态
                self.control_flow = ControlFlow::ThrownError {
                    error_type: error_name.clone(),
                    error_data: error_data.clone(),
                    line: statement.line,
                    column: statement.column,
                };

                return Err(KairoError::control_flow_signal(self.control_flow.clone(), "Error thrown"));
            }
        }
        Ok(())
    }

    pub(super) fn execute_block(&mut self, block: Block) -> Result<()> {
        for statement in block.statements { self.execute_statement(statement)?; if !matches!(self.control_flow, ControlFlow::None) { break; } }
        Ok(())
    }
}
