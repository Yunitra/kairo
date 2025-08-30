// SPDX-License-Identifier: Apache-2.0

//! 类型系统：类型枚举定义与显示实现
//!
//! 本模块仅包含 Kairo 语言中的类型枚举 `KairoType`，不包含值与运行时逻辑。
//! 与值相关的逻辑见 `types/value.rs`。

#[derive(Debug, Clone, PartialEq)]
pub enum KairoType {
    /// 整数类型
    Int,
    /// 浮点数类型
    Float,
    /// 文本字符串类型
    Text,
    /// 布尔类型
    Bool,
    /// 列表类型（单一元素类型）
    List(Box<KairoType>),
    /// 元组类型（逐位类型）
    Tuple(Vec<KairoType>),
    /// 映射类型（键类型, 值类型）。目前键在实现上固定为字符串，但类型系统保留键类型以便未来扩展。
    Map(Box<KairoType>, Box<KairoType>),
    /// 函数类型：参数类型列表 + 返回类型
    Function {
        params: Vec<KairoType>,
        return_type: Box<KairoType>,
    },
    /// 单元类型 ()
    Unit,
    /// 无返回（用于函数声明）
    Void,
    /// 泛型占位，如 T/K/V
    Generic(String),
    /// 可空类型，如 String?
    Nullable(Box<KairoType>),
    /// 错误类型
    Error {
        name: String,
        /// 错误数据字段类型（可选）
        fields: Option<Vec<(String, KairoType)>>,
    },
    /// 错误组类型
    ErrorGroup {
        name: String,
        /// 包含的错误类型名称
        errors: Vec<String>,
    },
}

impl std::fmt::Display for KairoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KairoType::Int => write!(f, "Int"),
            KairoType::Float => write!(f, "Float"),
            KairoType::Text => write!(f, "Text"),
            KairoType::Bool => write!(f, "Bool"),
            KairoType::List(inner) => write!(f, "List[{}]", inner),
            KairoType::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            KairoType::Map(key, value) => write!(f, "Map[{} -> {}]", key, value),
            KairoType::Function { params, return_type } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
            KairoType::Unit => write!(f, "()"),
            KairoType::Void => write!(f, "Void"),
            KairoType::Generic(name) => write!(f, "{}", name),
            KairoType::Nullable(inner) => write!(f, "{}?", inner),
            KairoType::Error { name, fields } => {
                if let Some(fields) = fields {
                    write!(f, "err {} {{ ", name)?;
                    for (i, (field_name, field_type)) in fields.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}: {}", field_name, field_type)?;
                    }
                    write!(f, " }}")
                } else {
                    write!(f, "err {}", name)
                }
            }
            KairoType::ErrorGroup { name, errors } => {
                write!(f, "err {} = ", name)?;
                for (i, error) in errors.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", error)?;
                }
                Ok(())
            }
        
        }
    }
}
