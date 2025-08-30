// SPDX-License-Identifier: Apache-2.0

//! 表达式求值

use std::collections::HashMap;

use crate::ast::{Expression, AssignmentOperator, BinaryOperator, UnaryOperator};
use crate::ast::expression::ExpressionKind;
use crate::error::{KairoError, Result};
use crate::types::{KairoType, KairoValue};

impl super::Interpreter {
    pub(super) fn evaluate_expression(&mut self, expression: Expression) -> Result<KairoValue> {
        match expression.kind {
            ExpressionKind::Literal(value) => Ok(value),
            ExpressionKind::Identifier(name) => {
                if let Some((value, _)) = self.get_variable(&name) { Ok(value) } else { Err(KairoError::runtime(format!("未定义的变量: {}", name), expression.line, expression.column)) }
            }
            ExpressionKind::Binary { left, operator, right } => { let left_val = self.evaluate_expression(*left)?; let right_val = self.evaluate_expression(*right)?; self.apply_binary_operator(operator, left_val, right_val, expression.line, expression.column) }
            ExpressionKind::Unary { operator, operand } => { let operand_val = self.evaluate_expression(*operand)?; self.apply_unary_operator(operator, operand_val, expression.line, expression.column) }
            ExpressionKind::FunctionCall { name, arguments } => { self.call_function(name, arguments, expression.line, expression.column) }
            ExpressionKind::MethodCall { object, method, arguments } => {
                let object_value = self.evaluate_expression(*object)?;
                let object_type = object_value.get_type();
                let mut function_names = Vec::new();
                function_names.push(format!("{}.{}", object_type, method));
                if matches!(object_type, KairoType::List(_)) { function_names.push(format!("List[T].{}", method)); }
                if matches!(object_type, KairoType::List(_)) { function_names.push(format!("List.{}", method)); }
                let mut last_error = None;
                for function_name in function_names {
                    let mut all_arguments = vec![Expression { kind: ExpressionKind::Literal(object_value.clone()), line: expression.line, column: expression.column }];
                    all_arguments.extend(arguments.clone());
                    match self.call_function(function_name, all_arguments, expression.line, expression.column) { Ok(result) => return Ok(result), Err(e) => last_error = Some(e), }
                }
                Err(last_error.unwrap_or_else(|| KairoError::runtime(format!("未知函数: {}.{}", object_type, method), expression.line, expression.column)))
            }
            ExpressionKind::List(elements) => { let mut values = Vec::new(); for element in elements { values.push(self.evaluate_expression(element)?); } Ok(KairoValue::List(values)) }
            ExpressionKind::Tuple(elements) => { let mut values = Vec::new(); for element in elements { values.push(self.evaluate_expression(element)?); } Ok(KairoValue::Tuple(values)) }
            ExpressionKind::Map(pairs) => {
                let mut map = HashMap::new();
                for (key_expr, value_expr) in pairs {
                    let key_val = self.evaluate_expression(key_expr)?;
                    let value_val = self.evaluate_expression(value_expr)?;
                    let key_str = match key_val {
                        KairoValue::Text(s) => s,
                        KairoValue::Int(i) => i.to_string(),
                        KairoValue::Float(f) => f.to_string(),
                        KairoValue::Bool(b) => b.to_string(),
                        _ => { return Err(KairoError::type_error("Map的键必须是Text、Int、Float或Bool类型".to_string(), expression.line, expression.column)); }
                    };
                    map.insert(key_str, value_val);
                }
                Ok(KairoValue::Map(map))
            }
            ExpressionKind::MatchExpr { value, arms } => {
                let matched_value = self.evaluate_expression(*value)?;
                for arm in arms {
                    if self.pattern_matches(&arm.pattern, &matched_value)? {
                        self.push_scope();
                        self.bind_pattern_variables(&arm.pattern, &matched_value, expression.line, expression.column)?;
                        if let Some(guard) = arm.guard.clone() { let guard_value = self.evaluate_expression(guard)?; if !guard_value.is_truthy() { self.pop_scope(); continue; } }
                        let result = self.evaluate_expression(arm.value)?;
                        self.pop_scope();
                        return Ok(result);
                    }
                }
                Ok(KairoValue::Unit)
            }
            ExpressionKind::Pipeline { left, right } => {
                let left_value = self.evaluate_expression(*left)?;
                match *right {
                    Expression { kind: ExpressionKind::FunctionCall { name, arguments }, .. } => {
                        let mut new_arguments = Vec::new();
                        for arg in arguments {
                            match arg {
                                Expression { kind: ExpressionKind::Placeholder, .. } => { new_arguments.push(Expression { kind: ExpressionKind::Literal(left_value.clone()), line: expression.line, column: expression.column }); }
                                _ => { new_arguments.push(arg); }
                            }
                        }
                        self.call_function(name, new_arguments, expression.line, expression.column)
                    }
                    _ => { Err(KairoError::syntax("管道操作符右边必须是函数调用".to_string(), expression.line, expression.column)) }
                }
            }
            ExpressionKind::Placeholder => { Err(KairoError::syntax("占位符只能在管道操作符中使用".to_string(), expression.line, expression.column)) }
            ExpressionKind::If { condition, then_branch, else_branch } => { let condition_value = self.evaluate_expression(*condition)?; if condition_value.is_truthy() { self.evaluate_expression(*then_branch) } else { self.evaluate_expression(*else_branch) } }
            ExpressionKind::Range { start, end, inclusive } => {
                let start_val = self.evaluate_expression(*start)?;
                let end_val = self.evaluate_expression(*end)?;
                match (start_val, end_val) {
                    (KairoValue::Int(start), KairoValue::Int(end)) => {
                        let mut values = Vec::new();
                        if inclusive { for i in start..=end { values.push(KairoValue::Int(i)); } } else { for i in start..end { values.push(KairoValue::Int(i)); } }
                        Ok(KairoValue::List(values))
                    }
                    _ => Err(KairoError::type_error("范围只支持整数".to_string(), expression.line, expression.column))
                }
            }
            ExpressionKind::Assignment { target, operator, value } => {
                let new_value = self.evaluate_expression(*value)?;
                match operator {
                    AssignmentOperator::Assign => {
                        if let Err(_) = self.update_variable(&target, new_value.clone(), expression.line, expression.column) {
                            return Err(KairoError::runtime(format!("尝试修改不可变变量: {}", target), expression.line, expression.column));
                        }
                    }
                    AssignmentOperator::AddAssign => {
                        if let Some((old_value, _)) = self.get_variable(&target) {
                            let result = self.apply_binary_operator(BinaryOperator::Add, old_value, new_value.clone(), expression.line, expression.column)?;
                            if let Err(_) = self.update_variable(&target, result, expression.line, expression.column) { return Err(KairoError::runtime(format!("尝试修改不可变变量: {}", target), expression.line, expression.column)); }
                        } else { return Err(KairoError::runtime(format!("未定义的变量: {}", target), expression.line, expression.column)); }
                    }
                    AssignmentOperator::SubAssign => {
                        if let Some((old_value, _)) = self.get_variable(&target) {
                            let result = self.apply_binary_operator(BinaryOperator::Subtract, old_value, new_value.clone(), expression.line, expression.column)?;
                            if let Err(_) = self.update_variable(&target, result, expression.line, expression.column) { return Err(KairoError::runtime(format!("尝试修改不可变变量: {}", target), expression.line, expression.column)); }
                        } else { return Err(KairoError::runtime(format!("未定义的变量: {}", target), expression.line, expression.column)); }
                    }
                    AssignmentOperator::MulAssign => {
                        if let Some((old_value, _)) = self.get_variable(&target) {
                            let result = self.apply_binary_operator(BinaryOperator::Multiply, old_value, new_value.clone(), expression.line, expression.column)?;
                            if let Err(_) = self.update_variable(&target, result, expression.line, expression.column) { return Err(KairoError::runtime(format!("尝试修改不可变变量: {}", target), expression.line, expression.column)); }
                        } else { return Err(KairoError::runtime(format!("未定义的变量: {}", target), expression.line, expression.column)); }
                    }
                    AssignmentOperator::DivAssign => {
                        if let Some((old_value, _)) = self.get_variable(&target) {
                            let result = self.apply_binary_operator(BinaryOperator::Divide, old_value, new_value.clone(), expression.line, expression.column)?;
                            if let Err(_) = self.update_variable(&target, result, expression.line, expression.column) { return Err(KairoError::runtime(format!("尝试修改不可变变量: {}", target), expression.line, expression.column)); }
                        } else { return Err(KairoError::runtime(format!("未定义的变量: {}", target), expression.line, expression.column)); }
                    }
                }
                Ok(new_value)
            }
            ExpressionKind::SafeCall { object, method, arguments } => {
                let object_value = self.evaluate_expression(*object)?;
                // 如果对象是 null，则安全调用返回 null
                if matches!(object_value, KairoValue::Null) {
                    return Ok(KairoValue::Null);
                }
                // 否则执行正常的方法调用
                let object_type = object_value.get_type();
                let mut function_names = Vec::new();
                function_names.push(format!("{}.{}", object_type, method));
                if matches!(object_type, KairoType::List(_)) { function_names.push(format!("List[T].{}", method)); }
                if matches!(object_type, KairoType::List(_)) { function_names.push(format!("List.{}", method)); }
                let mut last_error = None;
                for function_name in function_names {
                    let mut all_arguments = vec![Expression { kind: ExpressionKind::Literal(object_value.clone()), line: expression.line, column: expression.column }];
                    all_arguments.extend(arguments.clone());
                    match self.call_function(function_name, all_arguments, expression.line, expression.column) { Ok(result) => return Ok(result), Err(e) => last_error = Some(e), }
                }
                Err(last_error.unwrap_or_else(|| KairoError::runtime(format!("未知函数: {}.{}", object_type, method), expression.line, expression.column)))
            }
            ExpressionKind::Elvis { left, right } => {
                let left_value = self.evaluate_expression(*left)?;
                // 如果左值为 null 或 false，返回右值
                if matches!(left_value, KairoValue::Null) || !left_value.is_truthy() {
                    self.evaluate_expression(*right)
                } else {
                    Ok(left_value)
                }
            }
            ExpressionKind::Ternary { condition, then_expr, else_expr } => {
                let condition_value = self.evaluate_expression(*condition)?;
                if condition_value.is_truthy() {
                    self.evaluate_expression(*then_expr)
                } else {
                    self.evaluate_expression(*else_expr)
                }
            }
        }
    }

    pub(super) fn apply_binary_operator(&self, operator: BinaryOperator, left: KairoValue, right: KairoValue, line: usize, column: usize) -> Result<KairoValue> {
        match operator {
            BinaryOperator::Add => match (left, right) {
                (KairoValue::Int(a), KairoValue::Int(b)) => Ok(KairoValue::Int(a + b)),
                (KairoValue::Float(a), KairoValue::Float(b)) => Ok(KairoValue::Float(a + b)),
                (KairoValue::Int(a), KairoValue::Float(b)) => Ok(KairoValue::Float(a as f64 + b)),
                (KairoValue::Float(a), KairoValue::Int(b)) => Ok(KairoValue::Float(a + b as f64)),
                (KairoValue::Text(a), KairoValue::Text(b)) => Ok(KairoValue::Text(a + &b)),
                _ => Err(KairoError::type_error("加法运算符的操作数类型不匹配".to_string(), line, column)),
            },
            BinaryOperator::Subtract => match (left, right) {
                (KairoValue::Int(a), KairoValue::Int(b)) => Ok(KairoValue::Int(a - b)),
                (KairoValue::Float(a), KairoValue::Float(b)) => Ok(KairoValue::Float(a - b)),
                (KairoValue::Int(a), KairoValue::Float(b)) => Ok(KairoValue::Float(a as f64 - b)),
                (KairoValue::Float(a), KairoValue::Int(b)) => Ok(KairoValue::Float(a - b as f64)),
                _ => Err(KairoError::type_error("减法运算符的操作数类型不匹配".to_string(), line, column)),
            },
            BinaryOperator::Multiply => match (left, right) {
                (KairoValue::Int(a), KairoValue::Int(b)) => Ok(KairoValue::Int(a * b)),
                (KairoValue::Float(a), KairoValue::Float(b)) => Ok(KairoValue::Float(a * b)),
                (KairoValue::Int(a), KairoValue::Float(b)) => Ok(KairoValue::Float(a as f64 * b)),
                (KairoValue::Float(a), KairoValue::Int(b)) => Ok(KairoValue::Float(a * b as f64)),
                _ => Err(KairoError::type_error("乘法运算符的操作数类型不匹配".to_string(), line, column)),
            },
            BinaryOperator::Divide => match (left, right) {
                (KairoValue::Int(a), KairoValue::Int(b)) => { if b == 0 { Err(KairoError::runtime("除零错误".to_string(), line, column)) } else { Ok(KairoValue::Float(a as f64 / b as f64)) } }
                (KairoValue::Float(a), KairoValue::Float(b)) => { if b == 0.0 { Err(KairoError::runtime("除零错误".to_string(), line, column)) } else { Ok(KairoValue::Float(a / b)) } }
                (KairoValue::Int(a), KairoValue::Float(b)) => { if b == 0.0 { Err(KairoError::runtime("除零错误".to_string(), line, column)) } else { Ok(KairoValue::Float(a as f64 / b)) } }
                (KairoValue::Float(a), KairoValue::Int(b)) => { if b == 0 { Err(KairoError::runtime("除零错误".to_string(), line, column)) } else { Ok(KairoValue::Float(a / b as f64)) } }
                _ => Err(KairoError::type_error("除法运算符的操作数类型不匹配".to_string(), line, column)),
            },
            BinaryOperator::Equal => Ok(KairoValue::Bool(self.values_equal(&left, &right))),
            BinaryOperator::NotEqual => Ok(KairoValue::Bool(!self.values_equal(&left, &right))),
            BinaryOperator::Less => match (left, right) {
                (KairoValue::Int(a), KairoValue::Int(b)) => Ok(KairoValue::Bool(a < b)),
                (KairoValue::Float(a), KairoValue::Float(b)) => Ok(KairoValue::Bool(a < b)),
                (KairoValue::Int(a), KairoValue::Float(b)) => Ok(KairoValue::Bool((a as f64) < b)),
                (KairoValue::Float(a), KairoValue::Int(b)) => Ok(KairoValue::Bool(a < (b as f64))),
                _ => Err(KairoError::type_error("比较运算符的操作数类型不匹配".to_string(), line, column)),
            },
            BinaryOperator::Greater => match (left, right) {
                (KairoValue::Int(a), KairoValue::Int(b)) => Ok(KairoValue::Bool(a > b)),
                (KairoValue::Float(a), KairoValue::Float(b)) => Ok(KairoValue::Bool(a > b)),
                (KairoValue::Int(a), KairoValue::Float(b)) => Ok(KairoValue::Bool((a as f64) > b)),
                (KairoValue::Float(a), KairoValue::Int(b)) => Ok(KairoValue::Bool(a > (b as f64))),
                _ => Err(KairoError::type_error("比较运算符的操作数类型不匹配".to_string(), line, column)),
            },
            BinaryOperator::LessEqual => match (left, right) {
                (KairoValue::Int(a), KairoValue::Int(b)) => Ok(KairoValue::Bool(a <= b)),
                (KairoValue::Float(a), KairoValue::Float(b)) => Ok(KairoValue::Bool(a <= b)),
                (KairoValue::Int(a), KairoValue::Float(b)) => Ok(KairoValue::Bool((a as f64) <= b)),
                (KairoValue::Float(a), KairoValue::Int(b)) => Ok(KairoValue::Bool(a <= (b as f64))),
                _ => Err(KairoError::type_error("比较运算符的操作数类型不匹配".to_string(), line, column)),
            },
            BinaryOperator::GreaterEqual => match (left, right) {
                (KairoValue::Int(a), KairoValue::Int(b)) => Ok(KairoValue::Bool(a >= b)),
                (KairoValue::Float(a), KairoValue::Float(b)) => Ok(KairoValue::Bool(a >= b)),
                (KairoValue::Int(a), KairoValue::Float(b)) => Ok(KairoValue::Bool((a as f64) >= b)),
                (KairoValue::Float(a), KairoValue::Int(b)) => Ok(KairoValue::Bool(a >= (b as f64))),
                _ => Err(KairoError::type_error("比较运算符的操作数类型不匹配".to_string(), line, column)),
            },
            BinaryOperator::And => Ok(KairoValue::Bool(left.is_truthy() && right.is_truthy())),
            BinaryOperator::Or => Ok(KairoValue::Bool(left.is_truthy() || right.is_truthy())),
        }
    }

    pub(super) fn apply_unary_operator(&self, operator: UnaryOperator, operand: KairoValue, line: usize, column: usize) -> Result<KairoValue> {
        match operator {
            UnaryOperator::Minus => match operand { KairoValue::Int(i) => Ok(KairoValue::Int(-i)), KairoValue::Float(f) => Ok(KairoValue::Float(-f)), _ => Err(KairoError::type_error("负号运算符只能用于数值类型".to_string(), line, column)) },
            UnaryOperator::Not => Ok(KairoValue::Bool(!operand.is_truthy())),
        }
    }

    pub(super) fn types_match(&self, expected: &KairoType, actual: &KairoType) -> bool {
        match (expected, actual) {
            (KairoType::Int, KairoType::Int) => true,
            (KairoType::Float, KairoType::Float) => true,
            (KairoType::Text, KairoType::Text) => true,
            (KairoType::Bool, KairoType::Bool) => true,
            (KairoType::Unit, KairoType::Unit) => true,
            (KairoType::Void, KairoType::Void) => true,
            (KairoType::Generic(_), _) => true,
            (_, KairoType::Generic(_)) => true,
            (KairoType::List(expected_inner), KairoType::List(actual_inner)) => { self.types_match(expected_inner, actual_inner) }
            (KairoType::Tuple(expected_types), KairoType::Tuple(actual_types)) => { expected_types.len() == actual_types.len() && expected_types.iter().zip(actual_types.iter()).all(|(e, a)| self.types_match(e, a)) }
            (KairoType::Map(expected_key, expected_value), KairoType::Map(actual_key, actual_value)) => { self.types_match(expected_key, actual_key) && self.types_match(expected_value, actual_value) }
            (KairoType::Function { params: e_params, return_type: e_ret }, KairoType::Function { params: a_params, return_type: a_ret }) => { e_params.len() == a_params.len() && e_params.iter().zip(a_params.iter()).all(|(e, a)| self.types_match(e, a)) && self.types_match(e_ret, a_ret) }
            _ => false,
        }
    }

    pub(super) fn values_equal(&self, left: &KairoValue, right: &KairoValue) -> bool {
        match (left, right) {
            (KairoValue::Int(a), KairoValue::Int(b)) => a == b,
            (KairoValue::Float(a), KairoValue::Float(b)) => (a - b).abs() < f64::EPSILON,
            (KairoValue::Int(a), KairoValue::Float(b)) => (*a as f64 - b).abs() < f64::EPSILON,
            (KairoValue::Float(a), KairoValue::Int(b)) => (a - *b as f64).abs() < f64::EPSILON,
            (KairoValue::Text(a), KairoValue::Text(b)) => a == b,
            (KairoValue::Bool(a), KairoValue::Bool(b)) => a == b,
            (KairoValue::List(a), KairoValue::List(b)) => { a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y)) }
            (KairoValue::Tuple(a), KairoValue::Tuple(b)) => { a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y)) }
            (KairoValue::Unit, KairoValue::Unit) => true,
            (KairoValue::Null, KairoValue::Null) => true,
            _ => false,
        }
    }
}
