// SPDX-License-Identifier: Apache-2.0

//! 运行时值：包含 `KairoValue` 以及与类型间互操作的辅助方法。
//!
//! 说明：
//! - Map 的键实现使用 `String`，保持与解释器一致。
//! - `get_type` 会根据值推断类型（List/Map/Function 会组合类型信息）。
//! - `is_truthy` 定义了 Kairo 的真值语义。

use std::collections::HashMap;

use super::r#type::KairoType;

#[derive(Debug, Clone, PartialEq)]
pub enum KairoValue {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
    List(Vec<KairoValue>),
    Tuple(Vec<KairoValue>),
    Map(HashMap<String, KairoValue>),
    /// 函数描述信息（函数体由解释器管理，避免循环依赖）
    Function {
        name: String,
        params: Vec<(String, KairoType)>,
        return_type: KairoType,
    },
    Unit,
}

impl KairoValue {
    /// 返回值的静态类型信息
    pub fn get_type(&self) -> KairoType {
        match self {
            KairoValue::Int(_) => KairoType::Int,
            KairoValue::Float(_) => KairoType::Float,
            KairoValue::Text(_) => KairoType::Text,
            KairoValue::Bool(_) => KairoType::Bool,
            KairoValue::List(items) => {
                if items.is_empty() {
                    KairoType::List(Box::new(KairoType::Unit))
                } else {
                    KairoType::List(Box::new(items[0].get_type()))
                }
            }
            KairoValue::Tuple(items) => {
                KairoType::Tuple(items.iter().map(|v| v.get_type()).collect())
            }
            KairoValue::Map(map) => {
                if map.is_empty() {
                    KairoType::Map(Box::new(KairoType::Text), Box::new(KairoType::Unit))
                } else {
                    let value_type = map.values().next().unwrap().get_type();
                    KairoType::Map(Box::new(KairoType::Text), Box::new(value_type))
                }
            }
            KairoValue::Function { params, return_type, .. } => {
                KairoType::Function {
                    params: params.iter().map(|(_, t)| t.clone()).collect(),
                    return_type: Box::new(return_type.clone()),
                }
            }
            KairoValue::Unit => KairoType::Unit,
        }
    }

    /// 真值判断：用于逻辑运算与控制流
    pub fn is_truthy(&self) -> bool {
        match self {
            KairoValue::Bool(b) => *b,
            KairoValue::Int(i) => *i != 0,
            KairoValue::Float(f) => *f != 0.0,
            KairoValue::Text(s) => !s.is_empty(),
            KairoValue::List(l) => !l.is_empty(),
            KairoValue::Map(m) => !m.is_empty(),
            KairoValue::Tuple(t) => !t.is_empty(),
            KairoValue::Function { .. } => true,
            KairoValue::Unit => false,
        }
    }
}

impl std::fmt::Display for KairoValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KairoValue::Int(i) => write!(f, "{}", i),
            KairoValue::Float(fl) => write!(f, "{}", fl),
            KairoValue::Text(s) => write!(f, "{}", s),
            KairoValue::Bool(b) => write!(f, "{}", b),
            KairoValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            KairoValue::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            KairoValue::Map(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            KairoValue::Function { name, params, return_type, .. } => {
                write!(f, "fun {}(", name)?;
                for (i, (param_name, param_type)) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", param_name, param_type)?;
                }
                write!(f, ") -> {}", return_type)
            }
            KairoValue::Unit => write!(f, "()"),
        }
    }
}
