// SPDX-License-Identifier: Apache-2.0

//! 类型系统模块入口：拆分为类型与值两部分
//! 
//! - `type.rs`：类型定义
//! - `value.rs`：值定义

pub mod r#type;
pub mod value;

pub use r#type::KairoType;
pub use value::KairoValue;
