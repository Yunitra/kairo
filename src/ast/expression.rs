// SPDX-License-Identifier: Apache-2.0

//! 表达式节点及其运算符

use crate::types::KairoValue;
use super::common::{CatchClause, ErrorHandler, Block};

/// 统一的表达式节点，携带位置信息（行、列）
#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Minus,
    Not,
}

#[derive(Debug, Clone)]
pub enum AssignmentOperator {
    /// =
    Assign,
    /// +=
    AddAssign,
    /// -=
    SubAssign,
    /// *=
    MulAssign,
    /// /=
    DivAssign,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Literal(KairoValue),
    
    Identifier(String),

    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },

    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },

    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },

    List(Vec<Expression>),

    Tuple(Vec<Expression>),

    Map(Vec<(Expression, Expression)>),

    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },

    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },

    Assignment {
        target: String,
        operator: AssignmentOperator,
        value: Box<Expression>,
    },

    MatchExpr {
        value: Box<Expression>,
        arms: Vec<super::common::MatchArmExpr>,
    },

    Pipeline {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// 管道占位符 `_` 在表达式位置的表示
    Placeholder,

    /// 安全调用 obj?.method() 或 obj?.field
    SafeCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },

    /// Elvis 操作符 expr ?: defaultValue
    Elvis {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// 三目运算符 condition ? then_expr : else_expr
    Ternary {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    
    /// try-catch 表达式
    TryCatch {
        try_expr: Box<Expression>,
        catch_clauses: Vec<CatchClause>,
        /// 默认 catch 值（无具体错误类型的 catch）
        default_catch: Option<Box<Expression>>,
    },

    /// Block 表达式（用于 try { ... } 等场景）
    BlockExpr(Block),
    
    /// 错误传播操作符（expr!）
    ErrorPropagation {
        expression: Box<Expression>,
    },

    /// 错误处理语法糖（expr !: default）
    ErrorHandle {
        expression: Box<Expression>,
        handler: ErrorHandler,
    },
}

impl Expression {
    pub fn new(kind: ExpressionKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}
