use std::path::Path;

use anyhow::{bail, Result};

use crate::compiler::ast::Program;
use super::stmt;

/// 解析Kairo源代码为抽象语法树
/// 
/// # 参数
/// * `source` - 源代码字符串
/// * `_file` - 源文件路径（用于错误报告，当前未使用）
/// 
/// # 返回值
/// * `Result<Program>` - 解析成功返回Program AST，失败返回错误信息
/// 
/// # 功能
/// 1. 逐行解析源代码
/// 2. 跳过空行和注释行（以//开头）
/// 3. 尝试解析为打印语句或赋值语句
/// 4. 如果无法解析则返回语法错误
pub fn parse(source: &str, _file: &Path) -> Result<Program> {
    let mut statements = Vec::new();

    // 逐行解析源代码
    for (i, raw_line) in source.lines().enumerate() {
        let line_no = i + 1; // 行号从1开始
        let line_trim = raw_line.trim();
        
        // 跳过空行和注释行
        if line_trim.is_empty() || line_trim.starts_with("//") { 
            continue; 
        }

        // 尝试解析为打印语句
        if let Some(stmt) = stmt::parse_print(line_trim, line_no)? {
            statements.push(stmt);
            continue;
        }
        
        // 尝试解析为赋值语句（使用原始行，因为需要保留空格信息）
        if let Some(stmt) = stmt::parse_assign(raw_line, line_no)? {
            statements.push(stmt);
            continue;
        }

        // 如果都无法解析，返回语法错误
        bail!("语法错误：无法解析第 {line_no} 行：{raw_line}");
    }

    Ok(Program { statements })
}
