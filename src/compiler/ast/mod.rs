/// AST节点定义模块
/// 包含程序、语句、表达式等AST节点类型
pub mod node;

/// 源码位置信息模块
/// 提供源码位置和范围的定义
pub mod span;

/// 导出AST节点类型
/// 方便其他模块使用
pub use node::{Program, Stmt, Expr};

/// 导出源码位置类型
pub use span::{SourceSpan};
