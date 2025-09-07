use anyhow::{bail, Result};

use crate::compiler::ast::{Expr, SourceSpan};

/// 解析表达式字符串
/// 
/// # 参数
/// * `s` - 要解析的表达式字符串
/// * `line_no` - 行号（用于错误报告）
/// 
/// # 返回值
/// * `Result<Expr>` - 解析成功返回表达式AST，失败返回错误信息
/// 
/// # 功能
/// 支持左结合的加法运算：a + b + 1
/// 将表达式按+分割，递归解析每个部分
pub(crate) fn parse_expr(s: &str, line_no: usize) -> Result<Expr> {
    // 支持左结合的加法：a + b + 1
    let parts: Vec<&str> = s.split('+').map(|t| t.trim()).collect();
    
    // 如果没有+号，直接解析为原子表达式
    if parts.len() == 1 {
        return parse_atom(parts[0], line_no);
    }
    
    // 从第一个部分开始构建表达式
    let mut expr = parse_atom(parts[0], line_no)?;
    
    // 依次处理后续部分，构建左结合的加法表达式
    for part in parts.iter().skip(1) {
        let rhs = parse_atom(part, line_no)?;
        
        // 计算新表达式的源码范围
        let span = match (&expr, &rhs) {
            (Expr::StringLit(_, a), Expr::StringLit(_, b)) => SourceSpan::single_line(line_no, a.start.col, b.end.col),
            (Expr::StringLit(_, a), _) => *a,
            (Expr::IntLit(_, a), _) => *a,
            (Expr::Ident(_, a), _) => *a,
            (Expr::BinaryAdd(_, _, a), _) => *a,
        };
        
        expr = Expr::BinaryAdd(Box::new(expr), Box::new(rhs), span);
    }
    Ok(expr)
}

/// 解析原子表达式（不可再分割的基本表达式）
/// 
/// # 参数
/// * `s` - 要解析的原子表达式字符串
/// * `line_no` - 行号（用于错误报告）
/// 
/// # 返回值
/// * `Result<Expr>` - 解析成功返回表达式AST，失败返回错误信息
/// 
/// # 支持的原子表达式类型
/// 1. 字符串字面量："hello"
/// 2. 整数字面量：42, -10
/// 3. 标识符：变量名
fn parse_atom(s: &str, line_no: usize) -> Result<Expr> {
    // 解析字符串字面量："hello"
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return Ok(Expr::StringLit(
            s[1..s.len()-1].to_string(), // 去掉首尾的引号
            SourceSpan::single_line(line_no, 1, s.len())
        ));
    }
    
    // 解析整数字面量：42, -10
    if let Ok(v) = s.parse::<i64>() {
        return Ok(Expr::IntLit(
            v, 
            SourceSpan::single_line(line_no, 1, s.len())
        ));
    }
    
    // 解析标识符：变量名
    if is_ident(s) {
        return Ok(Expr::Ident(
            s.to_string(), 
            SourceSpan::single_line(line_no, 1, s.len())
        ));
    }
    
    // 如果都不匹配，返回语法错误
    bail!("语法错误：无法解析表达式 `{s}`（第 {line_no} 行）");
}

/// 检查字符串是否为有效的标识符
/// 
/// # 参数
/// * `s` - 要检查的字符串
/// 
/// # 返回值
/// * `bool` - 如果是有效标识符返回true，否则返回false
/// 
/// # 标识符规则
/// 1. 首字符必须是字母或下划线
/// 2. 后续字符可以是字母、数字或下划线
fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    
    // 检查首字符
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    
    // 检查后续字符
    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '_') { 
            return false; 
        }
    }
    true
}
