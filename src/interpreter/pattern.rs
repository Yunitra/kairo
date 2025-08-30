// SPDX-License-Identifier: Apache-2.0

//! 模式匹配与变量绑定

use crate::ast::Pattern;
use crate::error::Result;
use crate::types::KairoValue;

impl super::Interpreter {
    pub(super) fn pattern_matches(&mut self, pattern: &Pattern, value: &KairoValue) -> Result<bool> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Literal(pattern_value) => Ok(self.values_equal(pattern_value, value)),
            Pattern::Identifier(_) => Ok(true),
            Pattern::Range { start, end, inclusive } => {
                match (start, end, value) {
                    (KairoValue::Int(s), KairoValue::Int(e), KairoValue::Int(v)) => {
                        if *inclusive { Ok(*v >= *s && *v <= *e) } else { Ok(*v >= *s && *v < *e) }
                    }
                    _ => Ok(false),
                }
            }
            Pattern::In(expr) => {
                let range_value = self.evaluate_expression(expr.clone())?;
                match range_value { KairoValue::List(items) => Ok(items.iter().any(|item| self.values_equal(item, value))), _ => Ok(false) }
            }
            Pattern::Tuple(patterns) => {
                match value {
                    KairoValue::Tuple(values) => {
                        if patterns.len() != values.len() { return Ok(false); }
                        for (p, v) in patterns.iter().zip(values.iter()) { if !self.pattern_matches(p, v)? { return Ok(false); } }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Pattern::TypePattern { var: _, type_name } => {
                let matches = match (type_name.as_str(), value) {
                    ("Int", KairoValue::Int(_)) => true,
                    ("Float", KairoValue::Float(_)) => true,
                    ("Text", KairoValue::Text(_)) => true,
                    ("Bool", KairoValue::Bool(_)) => true,
                    ("List", KairoValue::List(_)) => true,
                    ("Map", KairoValue::Map(_)) => true,
                    _ => false,
                };
                Ok(matches)
            }
        }
    }

    pub(super) fn bind_pattern_variables(&mut self, pattern: &Pattern, value: &KairoValue, line: usize, column: usize) -> Result<()> {
        match pattern {
            Pattern::Identifier(name) => { self.set_variable(name.clone(), value.clone(), false, line, column)?; }
            Pattern::TypePattern { var, type_name: _ } => { self.set_variable(var.clone(), value.clone(), false, line, column)?; }
            Pattern::Tuple(patterns) => {
                if let KairoValue::Tuple(values) = value { for (p, v) in patterns.iter().zip(values.iter()) { self.bind_pattern_variables(p, v, line, column)?; } }
            }
            _ => {}
        }
        Ok(())
    }
}
