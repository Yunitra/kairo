// SPDX-License-Identifier: Apache-2.0

//! 控制流信号

use crate::types::KairoValue;

#[derive(Debug, Clone)]
pub enum ControlFlow {
    None,
    Break(usize),
    Continue,
    /// return 值，及其出现位置（用于错误定位）
    Return(KairoValue, usize, usize),
    /// 错误值，及其出现位置（用于错误传播）
    Error(KairoValue, usize, usize),
    /// 抛出的错误，包含错误类型信息
    ThrownError {
        error_type: String,
        error_data: Option<std::collections::HashMap<String, KairoValue>>,
        line: usize,
        column: usize,
    },
}
