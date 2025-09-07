/// Rust代码生成实现模块
/// 包含将AST转换为Rust代码的具体实现
pub mod imp;

/// 导出Rust代码生成函数
pub use imp::generate_rust;
