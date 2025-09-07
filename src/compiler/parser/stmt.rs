use anyhow::{bail, Result};

use crate::compiler::ast::{SourceSpan, Stmt};

use super::expr;

/// 解析打印语句
/// 
/// # 参数
/// * `line` - 要解析的行（已去除首尾空格）
/// * `line_no` - 行号（用于错误报告）
/// 
/// # 返回值
/// * `Result<Option<Stmt>>` - 如果是打印语句返回Some(Stmt::Print)，否则返回None
/// 
/// # 语法格式
/// print("字符串内容")
/// 
/// # 限制
/// 目前仅支持简单的字符串字面量，不支持转义字符
pub(crate) fn parse_print(line: &str, line_no: usize) -> Result<Option<Stmt>> {
    // 检查是否为print语句格式
    if !line.starts_with("print(") || !line.ends_with(")") { 
        return Ok(None); 
    }
    
    // 提取括号内的内容
    let inner = &line[6..line.len()-1];
    let inner = inner.trim();
    
    // 仅支持简单的字符串字面量："..."
    if !(inner.starts_with('"') && inner.ends_with('"') && inner.len() >= 2) {
        bail!("语法错误：print(...) 仅支持字符串字面量，第 {line_no} 行");
    }
    
    // TODO: 支持转义字符
    let content = inner[1..inner.len()-1].to_string(); // 去掉首尾引号
    let _span = SourceSpan::single_line(line_no, 1, line.len());
    Ok(Some(Stmt::Print { content, _span }))
}

/// 解析赋值语句
/// 
/// # 参数
/// * `raw` - 原始行内容（保留空格信息）
/// * `line_no` - 行号（用于错误报告）
/// 
/// # 返回值
/// * `Result<Option<Stmt>>` - 如果是赋值语句返回Some(Stmt::Assign)，否则返回None
/// 
/// # 语法格式
/// 变量名 = 表达式        // 不可变变量赋值
/// $变量名 = 表达式       // 可变变量声明和赋值
/// 变量名 = 表达式        // 已存在变量的重新赋值
pub(crate) fn parse_assign(raw: &str, line_no: usize) -> Result<Option<Stmt>> {
    // 快速路径：如果没有=号，则不是赋值语句
    let Some((lhs_raw, rhs_raw)) = raw.split_once('=') else { 
        return Ok(None); 
    };

    // 确定是否为可变声明并找到标识符
    let mut i = 0usize;
    let bytes = lhs_raw.as_bytes();
    
    // 跳过前导空格
    while i < bytes.len() && bytes[i].is_ascii_whitespace() { 
        i += 1; 
    }
    
    // 检查是否有$前缀（可变声明）
    let mut decl_mut = false;
    if i < bytes.len() && bytes[i] == b'$' {
        decl_mut = true;
        i += 1;
        // 跳过$后的空格
        while i < bytes.len() && bytes[i].is_ascii_whitespace() { 
            i += 1; 
        }
    }
    
    let name_start = i; // 变量名开始的字节偏移
    
    // 解析标识符
    if i >= bytes.len() { 
        return Ok(None); 
    }
    
    let mut j = i;
    let first = bytes[j] as char;
    
    // 首字符必须是字母或下划线
    if !(first.is_ascii_alphabetic() || first == '_') { 
        return Ok(None); 
    }
    
    j += 1;
    // 继续解析标识符的其余部分
    while j < bytes.len() {
        let c = bytes[j] as char;
        if c.is_ascii_alphanumeric() || c == '_' { 
            j += 1; 
        } else { 
            break; 
        }
    }
    
    let name = &lhs_raw[name_start..j];
    
    // 确保左值剩余部分只有空格
    let rest = &lhs_raw[j..];
    if rest.trim() != "" {
        bail!("语法错误：无效的左值 `{}`（第 {line_no} 行）", lhs_raw.trim());
    }

    // 解析右值表达式
    let expr = expr::parse_expr(rhs_raw.trim(), line_no)?;
    let span = SourceSpan::single_line(line_no, 1, raw.len());
    
    // 列号从1开始；长度为字节数（简化处理，假设ASCII）
    let name_span = SourceSpan::single_line(line_no, name_start + 1, name_start + name.len() + 1);
    
    Ok(Some(Stmt::Assign { 
        name: name.to_string(), 
        decl_mut, 
        expr, 
        span, 
        name_span 
    }))
}
