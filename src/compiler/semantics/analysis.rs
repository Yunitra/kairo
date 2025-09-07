use std::collections::HashMap;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::compiler::ast::{Expr, Program, SourceSpan, Stmt};
use super::diagnostics::{caret_line, get_line, render_error};

/// 变量的可变性类型
/// 
/// # 变体
/// * `Immutable` - 不可变变量（默认）
/// * `Mutable` - 可变变量（使用$前缀声明）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability { 
    /// 不可变变量
    Immutable, 
    /// 可变变量
    Mutable 
}

/// 语义分析信息
/// 包含程序中的所有变量及其可变性信息
#[derive(Debug, Default)]
pub struct SemanticInfo {
    /// 变量名到可变性的映射表
    pub vars: HashMap<String, Mutability>,
}

/// 执行语义检查（不可变性规则）并构建符号表
/// 
/// # 参数
/// * `program` - 程序的抽象语法树
/// * `file` - 源文件路径（用于错误报告）
/// * `source` - 源代码字符串（用于错误报告）
/// 
/// # 返回值
/// * `Result<SemanticInfo>` - 语义分析成功返回符号表，失败返回错误信息
/// 
/// # 检查规则
/// 1. 变量声明规则：
///    - $变量名 = 值：声明可变变量，不能重复声明
///    - 变量名 = 值：声明不可变变量或重新赋值
/// 2. 不可变性规则：
///    - 不可变变量不能重新赋值
///    - 可变变量可以重新赋值
/// 3. 未定义变量检查：
///    - 表达式中使用的变量必须已声明
pub fn check_semantics(program: &Program, file: &Path, source: &str) -> Result<SemanticInfo> {
    let mut info = SemanticInfo::default();
    let mut errors: Vec<String> = Vec::new();

    // 第一遍：处理声明和可变性规则
    for stmt in &program.statements {
        match stmt {
            Stmt::Print { .. } => {
                // 打印语句不需要语义检查
            }
            Stmt::Assign { name, decl_mut, span: _span, name_span, .. } => {
                let existed = info.vars.get(name).cloned();
                
                if *decl_mut {
                    // 处理可变变量声明（$前缀）
                    match existed {
                        None => { 
                            // 新声明，添加到符号表
                            info.vars.insert(name.clone(), Mutability::Mutable); 
                        }
                        Some(_) => {
                            // 重复声明，报告错误
                            errors.push(friendly_error_redeclare(file, source, name, *name_span));
                        }
                    }
                } else {
                    // 处理不可变变量赋值
                    match existed {
                        None => { 
                            // 新声明，添加到符号表
                            info.vars.insert(name.clone(), Mutability::Immutable); 
                        }
                        Some(Mutability::Immutable) => {
                            // 试图修改不可变变量，报告错误
                            errors.push(friendly_error_assign_immutable(file, source, name, *name_span));
                        }
                        Some(Mutability::Mutable) => { 
                            // 修改可变变量，允许
                        }
                    }
                }
            }
        }
    }

    // 第二遍：检查表达式中未定义的变量
    let mut declared: HashMap<&str, Mutability> = HashMap::new();
    for stmt in &program.statements {
        match stmt {
            Stmt::Print { .. } => {
                // 打印语句不需要检查
            }
            Stmt::Assign { name, decl_mut, expr, name_span: _name_span, .. } => {
                // 检查表达式中使用的变量是否已声明
                collect_undefined_idents(expr, &declared, file, source, &mut errors);
                
                // 更新已声明变量列表
                if *decl_mut {
                    declared.insert(name.as_str(), Mutability::Mutable);
                } else if !declared.contains_key(name.as_str()) {
                    declared.insert(name.as_str(), Mutability::Immutable);
                }
            }
        }
    }

    // 如果有错误，返回所有错误信息
    if !errors.is_empty() {
        return Err(anyhow!(errors.join("\n")));
    }

    Ok(info)
}

/// 生成修改不可变变量的友好错误信息
/// 
/// # 参数
/// * `file` - 源文件路径
/// * `source` - 源代码字符串
/// * `name` - 变量名
/// * `name_span` - 变量名的源码位置
/// 
/// # 返回值
/// * `String` - 格式化的错误信息
fn friendly_error_assign_immutable(
    file: &Path,
    source: &str,
    name: &str,
    name_span: SourceSpan,
) -> String {
    let filename = file.file_name().and_then(|s| s.to_str()).unwrap_or("<unknown>");
    let line_no = name_span.start.line;
    let col = name_span.start.col;
    let line_text = get_line(source, line_no);
    let caret = caret_line(name_span);
    let summary = format!("你试图修改不可变变量 `{name}`");
    let suggestions = format!(
        "   - 如果你想让它可变，请在首次赋值时加 `$`：\n        ${name} = 0   ← 这样声明\n        {name} = {name} + 1   ← 这样修改\n   - 或者，你是否想创建一个新变量？\n        new_{name} = {name} + 1",
    );
    render_error(&summary, filename, line_no, col, &line_text, &caret, &suggestions)
}

/// 生成重复声明变量的友好错误信息
/// 
/// # 参数
/// * `file` - 源文件路径
/// * `source` - 源代码字符串
/// * `name` - 变量名
/// * `name_span` - 变量名的源码位置
/// 
/// # 返回值
/// * `String` - 格式化的错误信息
fn friendly_error_redeclare(file: &Path, source: &str, name: &str, name_span: SourceSpan) -> String {
    let filename = file.file_name().and_then(|s| s.to_str()).unwrap_or("<unknown>");
    let line_no = name_span.start.line;
    let col = name_span.start.col;
    let line_text = get_line(source, line_no);
    let caret = caret_line(name_span);
    let summary = format!("变量 `{name}` 已在之前声明，不能重复声明");
    let suggestions = format!(
        "   - 如需重新赋值，请直接写：\n        {name} = ...\n   - 如需新变量，请改用不同的名称：\n        {name}_2 = ...",
    );
    render_error(&summary, filename, line_no, col, &line_text, &caret, &suggestions)
}

/// 递归收集表达式中未定义的标识符
/// 
/// # 参数
/// * `expr` - 要检查的表达式
/// * `declared` - 已声明的变量映射表
/// * `file` - 源文件路径
/// * `source` - 源代码字符串
/// * `errors` - 错误信息列表（用于收集错误）
/// 
/// # 功能
/// 遍历表达式树，检查所有标识符是否已在之前声明
/// 对于未定义的变量，生成友好的错误信息并添加到错误列表
fn collect_undefined_idents(
    expr: &Expr,
    declared: &HashMap<&str, Mutability>,
    file: &Path,
    source: &str,
    errors: &mut Vec<String>,
) {
    match expr {
        Expr::Ident(name, span) => {
            // 检查标识符是否已声明
            if !declared.contains_key(name.as_str()) {
                let filename = file.file_name().and_then(|s| s.to_str()).unwrap_or("<unknown>");
                let line_no = span.start.line;
                let line_text = get_line(source, line_no);
                
                // 尝试在行中定位标识符以获得更准确的列位置
                let (col, span_for_caret) = if let Some(idx) = line_text.find(name) {
                    let start_col = idx + 1; // 转换为1基索引
                    let end_col = start_col + name.len();
                    (start_col, SourceSpan::single_line(line_no, start_col, end_col))
                } else {
                    (span.start.col, *span)
                };
                
                let caret = caret_line(span_for_caret);
                let summary = format!("使用了未定义的变量 `{name}`");
                let suggestions = format!(
                    "   - 请先声明变量：\n        {name} = ...    // 不可变\n        ${name} = ...   // 可变",
                );
                errors.push(render_error(&summary, filename, line_no, col, &line_text, &caret, &suggestions));
            }
        }
        Expr::BinaryAdd(a, b, _) => {
            // 递归检查二元加法表达式的左右操作数
            collect_undefined_idents(a, declared, file, source, errors);
            collect_undefined_idents(b, declared, file, source, errors);
        }
        _ => {
            // 其他表达式类型（字面量等）不需要检查
        }
    }
}
