/// 诊断信息模块
/// 提供错误报告和源码位置标记功能
pub mod diagnostics;

/// 语义分析模块
/// 执行变量声明检查、不可变性规则验证等语义分析
pub mod analysis;

/// 导出语义分析的主要类型和函数
pub use analysis::{check_semantics, Mutability, SemanticInfo};
