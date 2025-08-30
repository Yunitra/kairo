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
}
