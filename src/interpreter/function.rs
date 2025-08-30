// SPDX-License-Identifier: Apache-2.0

//! 函数定义与调用逻辑

use crate::ast::{Expression, Block};
use crate::types::{KairoType, KairoValue};
use crate::error::{KairoError, Result};

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<(String, KairoType, bool, Option<Expression>, bool)>,
    pub return_type: Option<KairoType>,
    pub body: FunctionBodyType,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum FunctionBodyType {
    Block(Block),
    Expression(Expression),
}

impl super::Interpreter {
    pub(super) fn call_function(&mut self, name: String, arguments: Vec<Expression>, line: usize, column: usize) -> Result<KairoValue> {
        // 内置函数
        match name.as_str() {
            "print" => {
                for arg in arguments { let value = self.evaluate_expression(arg)?; print!("{}", value); }
                println!();
                return Ok(KairoValue::Unit);
            }
            _ => {}
        }

        if let Some(function) = self.get_function(&name) {
            let mut arg_values = Vec::new();
            for arg in arguments { arg_values.push(self.evaluate_expression(arg)?); }

            let mut final_arg_values = arg_values.clone();
            let has_variadic = function.parameters.iter().any(|(_, _, _, _, variadic)| *variadic);
            if has_variadic {
                let mut processed_args = Vec::new();
                let mut arg_index = 0;
                for (_param_name, _param_type, _mutable, default_expr, variadic) in &function.parameters {
                    if *variadic {
                        let mut variadic_args = Vec::new();
                        while arg_index < final_arg_values.len() { variadic_args.push(final_arg_values[arg_index].clone()); arg_index += 1; }
                        processed_args.push(KairoValue::List(variadic_args));
                    } else {
                        if arg_index < final_arg_values.len() { processed_args.push(final_arg_values[arg_index].clone()); arg_index += 1; }
                        else if let Some(default_expr) = default_expr { let default_value = self.evaluate_expression(default_expr.clone())?; processed_args.push(default_value); }
                        else { return Err(KairoError::runtime(format!("函数 {} 的第 {} 个参数 没有默认值", name, processed_args.len() + 1), line, column)); }
                    }
                }
                final_arg_values = processed_args;
            } else {
                while final_arg_values.len() < function.parameters.len() {
                    let param_index = final_arg_values.len();
                    let (_param_name, _t, _m, default_expr, _) = &function.parameters[param_index];
                    if let Some(default_expr) = default_expr { let default_value = self.evaluate_expression(default_expr.clone())?; final_arg_values.push(default_value); }
                    else { return Err(KairoError::runtime(format!("函数 {} 的第 {} 个参数 没有默认值", name, param_index + 1), line, column)); }
                }
                if final_arg_values.len() != function.parameters.len() {
                    return Err(KairoError::runtime(format!("函数 {} 期望 {} 个参数，但得到了 {}", name, function.parameters.len(), final_arg_values.len()), line, column));
                }
            }

            // 类型检查
            for (i, ((param_name, param_type, _, _, variadic), arg_value)) in function.parameters.iter().zip(final_arg_values.iter()).enumerate() {
                if *variadic {
                    if !matches!(arg_value, KairoValue::List(_)) {
                        return Err(KairoError::runtime(format!("函数 {} 的可变参数 {} 必须是列表类型", name, param_name), line, column));
                    }
                } else {
                    let arg_type = arg_value.get_type();
                    if !self.types_match(param_type, &arg_type) {
                        return Err(KairoError::runtime(format!("函数 {} 的第 {} 个参数 {} 类型不匹配: 期望 {}, 得到 {}", name, i + 1, param_name, param_type, arg_type), line, column));
                    }
                }
            }

            self.push_scope();
            for ((param_name, _t, mutable, _d, _v), arg_value) in function.parameters.iter().zip(final_arg_values.iter()) {
                self.set_variable(param_name.clone(), arg_value.clone(), *mutable, line, column)?;
            }

            let (result, return_line, return_column) = match &function.body {
                FunctionBodyType::Block(block) => {
                    self.execute_block(block.clone())?;
                    match std::mem::replace(&mut self.control_flow, super::control_flow::ControlFlow::None) {
                        super::control_flow::ControlFlow::Return(value, return_line, return_column) => (value, return_line, return_column),
                        _ => {
                            match &function.return_type {
                                Some(KairoType::Void) | None => (KairoValue::Unit, function.line, function.column),
                                Some(return_type) => {
                                    return Err(KairoError::runtime(format!("函数 {} 期望返回 {} 类型，但没有返回语句", name, return_type), function.line, function.column));
                                }
                            }
                        }
                    }
                }
                FunctionBodyType::Expression(expr) => { let result = self.evaluate_expression(expr.clone())?; (result, function.line, function.column) }
            };

            self.pop_scope();
            if let Some(expected_return_type) = &function.return_type {
                if !matches!(expected_return_type, KairoType::Void) {
                    let actual_return_type = result.get_type();
                    if !self.types_match(expected_return_type, &actual_return_type) {
                        return Err(KairoError::runtime(format!("函数 {} 返回类型不匹配: 期望 {}, 得到 {}", name, expected_return_type, actual_return_type), return_line, return_column));
                    }
                }
            }
            Ok(result)
        } else {
            Err(KairoError::runtime(format!("未知函数: {}", name), line, column))
        }
    }
}


