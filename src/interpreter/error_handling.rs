// SPDX-License-Identifier: Apache-2.0

//! 错误处理系统
//!
//! 负责错误类型注册、错误匹配、错误值创建等功能

use crate::ast::ErrorField;
use crate::error::KairoError;
use crate::types::KairoValue;
use super::ControlFlow;
use std::collections::HashMap;

/// 错误处理系统
pub struct ErrorHandlingSystem {
    /// 错误类型定义注册表
    pub(super) error_types: HashMap<String, Option<Vec<ErrorField>>>,

    /// 错误组定义注册表
    pub(super) error_groups: HashMap<String, Vec<String>>,
}

impl ErrorHandlingSystem {
    pub fn new() -> Self {
        Self {
            error_types: HashMap::new(),
            error_groups: HashMap::new(),
        }
    }

    /// 注册错误类型
    pub fn register_error_type(&mut self, name: String, fields: Option<Vec<ErrorField>>, line: usize, column: usize) -> Result<(), KairoError> {
        if self.error_types.contains_key(&name) {
            return Err(KairoError::runtime(
                format!("错误类型 {} 已经定义", name),
                line,
                column,
            ));
        }
        self.error_types.insert(name, fields);
        Ok(())
    }

    /// 注册错误组
    pub fn register_error_group(&mut self, name: String, errors: Vec<String>, line: usize, column: usize) -> Result<(), KairoError> {
        if self.error_groups.contains_key(&name) {
            return Err(KairoError::runtime(
                format!("错误组 {} 已经定义", name),
                line,
                column,
            ));
        }
        
        // 验证错误组中的所有错误类型都已定义
        for error_name in &errors {
            if !self.error_types.contains_key(error_name) {
                return Err(KairoError::runtime(
                    format!("错误类型 {} 未定义", error_name),
                    line,
                    column,
                ));
            }
        }
        
        self.error_groups.insert(name, errors);
        Ok(())
    }

    /// 检查错误类型是否已定义
    pub fn is_error_type_defined(&self, name: &str) -> bool {
        self.error_types.contains_key(name)
    }

    /// 检查错误是否属于某个错误组
    pub fn is_error_in_group(&self, error_name: &str, group_name: &str) -> bool {
        if let Some(group_errors) = self.error_groups.get(group_name) {
            group_errors.contains(&error_name.to_string())
        } else {
            false
        }
    }

    /// 检查错误类型是否匹配（包括错误组）
    pub fn is_error_type_match(&self, error: &KairoError, expected_type: &str) -> bool {
        // 从错误消息中提取错误类型
        // 错误消息格式应该是 "错误类型: 描述"
        if let Some(colon_pos) = error.message.find(':') {
            let actual_type = error.message[..colon_pos].trim();
            return actual_type == expected_type || self.is_error_in_group(actual_type, expected_type);
        }
        
        // 如果没有冒号，检查整个消息是否匹配
        error.message == expected_type || self.is_error_in_group(&error.message, expected_type)
    }

    /// 从错误创建错误值
    pub fn create_error_value_from_error(&self, error: &KairoError, error_type: &str) -> KairoValue {
        // 从错误消息中提取错误数据
        let mut error_data = None;
        if let Some(colon_pos) = error.message.find(':') {
            let description = error.message[colon_pos + 1..].trim();
            if !description.is_empty() {
                let mut data = std::collections::HashMap::new();
                data.insert("message".to_string(), KairoValue::Text(description.to_string()));
                error_data = Some(data);
            }
        }
        
        KairoValue::Error {
            name: error_type.to_string(),
            data: error_data,
        }
    }

    /// 检查控制流中的错误类型是否匹配
    pub fn is_control_flow_error_match(&self, control_flow: &ControlFlow, expected_type: &str) -> bool {
        match control_flow {
            ControlFlow::ThrownError { error_type, .. } => {
                error_type == expected_type || self.is_error_in_group(error_type, expected_type)
            }
            _ => false,
        }
    }

    /// 从控制流错误创建错误值
    pub fn create_error_value_from_control_flow(&self, control_flow: &ControlFlow, error_type: &str) -> KairoValue {
        match control_flow {
            ControlFlow::ThrownError { error_type: _actual_type, error_data, .. } => {
                KairoValue::Error {
                    name: error_type.to_string(),
                    data: error_data.clone(),
                }
            }
            _ => KairoValue::Error {
                name: error_type.to_string(),
                data: None,
            },
        }
    }
}
