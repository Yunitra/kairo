use super::span::SourceSpan;

/// Kairo程序的抽象语法树根节点
/// 包含程序中的所有语句
#[derive(Debug, Clone)]
pub struct Program {
    /// 程序中的语句列表
    pub statements: Vec<Stmt>,
}

/// 语句类型
/// 表示Kairo语言中的各种语句
#[derive(Debug, Clone)]
pub enum Stmt {
    /// 打印语句：print("内容")
    /// 
    /// # 字段
    /// * `content` - 要打印的字符串内容
    /// * `_span` - 源码位置信息（用于错误报告）
    Print { content: String, _span: SourceSpan },
    
    /// 赋值语句：变量名 = 表达式 或 $变量名 = 表达式
    /// 
    /// # 字段
    /// * `name` - 变量名
    /// * `decl_mut` - 是否为可变变量声明（$前缀）
    /// * `expr` - 赋值的表达式
    /// * `span` - 整个语句的源码位置
    /// * `name_span` - 变量名的源码位置
    Assign { name: String, decl_mut: bool, expr: Expr, span: SourceSpan, name_span: SourceSpan },
}

/// 表达式类型
/// 表示Kairo语言中的各种表达式
#[derive(Debug, Clone)]
pub enum Expr {
    /// 字符串字面量："hello world"
    /// 
    /// # 字段
    /// * `String` - 字符串内容（不包含引号）
    /// * `SourceSpan` - 源码位置信息
    StringLit(String, SourceSpan),
    
    /// 整数字面量：42, -10
    /// 
    /// # 字段
    /// * `i64` - 整数值
    /// * `SourceSpan` - 源码位置信息
    IntLit(i64, SourceSpan),
    
    /// 标识符：变量名
    /// 
    /// # 字段
    /// * `String` - 变量名
    /// * `SourceSpan` - 源码位置信息
    Ident(String, SourceSpan),
    
    /// 二元加法表达式：a + b
    /// 
    /// # 字段
    /// * `Box<Expr>` - 左操作数
    /// * `Box<Expr>` - 右操作数
    /// * `SourceSpan` - 源码位置信息
    BinaryAdd(Box<Expr>, Box<Expr>, SourceSpan),
}
