use std::collections::HashMap;

use anyhow::Result;

use crate::compiler::ast::{Expr, Program, Stmt};
use crate::compiler::semantics::{Mutability, SemanticInfo};

/// 将Kairo程序转换为Rust代码
/// 
/// # 参数
/// * `program` - Kairo程序的抽象语法树
/// * `semantic` - 语义分析信息（包含变量可变性信息）
/// 
/// # 返回值
/// * `Result<String>` - 生成的Rust源代码字符串
/// 
/// # 转换规则
/// 1. 不可变变量：直接转换为Rust的let绑定
/// 2. 可变变量：使用Rc<RefCell<T>>实现可变性
/// 3. 打印语句：转换为println!宏调用
/// 4. 表达式：递归转换各种表达式类型
pub fn generate_rust(program: &Program, semantic: &SemanticInfo) -> Result<String> {
    let mut out = String::new();
    
    // 检查是否需要可变性支持，如果需要则导入相关模块
    let needs_rc = semantic.vars.values().any(|m| matches!(m, Mutability::Mutable));
    if needs_rc {
        out.push_str("use std::rc::Rc;\n");
        out.push_str("use std::cell::RefCell;\n\n");
    }
    
    out.push_str("fn main() {\n");

    // 跟踪已声明的变量，用于决定是使用let声明还是赋值
    let mut declared: HashMap<&str, bool> = HashMap::new();

    // 遍历所有语句并转换为Rust代码
    for stmt in &program.statements {
        match stmt {
            Stmt::Print { content, .. } => {
                // 转换打印语句为println!宏
                out.push_str(&format!("    println!(\"{}\");\n", escape(content)));
            }
            Stmt::Assign { name, decl_mut, expr, .. } => {
                // 获取变量的可变性信息
                let mutability = semantic.vars.get(name).cloned().unwrap_or(Mutability::Immutable);
                let is_first = !declared.contains_key(name.as_str());
                let expr_code = gen_expr(expr, &semantic.vars);
                
                // 根据变量状态生成不同的Rust代码
                match (is_first, mutability, *decl_mut) {
                    // 首次声明可变变量
                    (true, Mutability::Mutable, true) => {
                        out.push_str(&format!("    let {} = Rc::new(RefCell::new({}));\n", name, expr_code));
                        declared.insert(name, true);
                    }
                    // 首次声明不可变变量
                    (true, Mutability::Immutable, false) => {
                        out.push_str(&format!("    let {} = {};\n", name, expr_code));
                        declared.insert(name, true);
                    }
                    // 修改已存在的可变变量
                    (false, Mutability::Mutable, _) => {
                        out.push_str(&format!("    *{}.borrow_mut() = {};\n", name, expr_code));
                    }
                    // 修改不可变变量（语义分析应该已阻止，但保留安全默认值）
                    (false, Mutability::Immutable, _) => {
                        out.push_str(&format!("    let {} = {}; // (note) immutable redeclaration fallback\n", name, expr_code));
                    }
                    // 语义不一致的情况（不应该发生，但保留安全默认值）
                    (true, Mutability::Mutable, false) | (true, Mutability::Immutable, true) => {
                        out.push_str(&format!("    let {} = {};\n", name, expr_code));
                        declared.insert(name, true);
                    }
                }
            }
        }
    }

    out.push_str("}\n");
    Ok(out)
}

/// 将表达式转换为Rust代码
/// 
/// # 参数
/// * `expr` - 要转换的表达式
/// * `vars` - 变量可变性映射表
/// 
/// # 返回值
/// * `String` - 生成的Rust表达式代码
/// 
/// # 转换规则
/// 1. 字符串字面量：添加引号并转义特殊字符
/// 2. 整数字面量：直接转换为字符串
/// 3. 标识符：根据可变性决定是否使用borrow()
/// 4. 二元加法：递归转换左右操作数
fn gen_expr(expr: &Expr, vars: &HashMap<String, Mutability>) -> String {
    match expr {
        Expr::StringLit(s, _) => {
            // 字符串字面量：添加引号并转义
            format!("\"{}\"", escape(s))
        }
        Expr::IntLit(v, _) => {
            // 整数字面量：直接转换
            v.to_string()
        }
        Expr::Ident(name, _) => {
            // 标识符：根据可变性决定访问方式
            match vars.get(name) {
                Some(Mutability::Mutable) => {
                    // 可变变量：使用borrow()获取值
                    format!("*{}.borrow()", name)
                }
                _ => {
                    // 不可变变量：直接使用
                    name.clone()
                }
            }
        }
        Expr::BinaryAdd(a, b, _) => {
            // 二元加法：递归转换左右操作数
            format!("({} + {})", gen_expr(a, vars), gen_expr(b, vars))
        }
    }
}

/// 转义字符串中的特殊字符
/// 
/// # 参数
/// * `s` - 要转义的字符串
/// 
/// # 返回值
/// * `String` - 转义后的字符串
/// 
/// # 转义规则
/// * `\` -> `\\`
/// * `"` -> `\"`
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
