// SPDX-License-Identifier: Apache-2.0

//! 表达式节点及其运算符

use crate::types::KairoValue;

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
pub enum Expression {
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
        line: usize,
        column: usize,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
        line: usize,
        column: usize,
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
        line: usize,
        column: usize,
    },
    MatchExpr {
        value: Box<Expression>,
        arms: Vec<super::common::MatchArmExpr>,
    },
    Pipeline {
        left: Box<Expression>,
        right: Box<Expression>,
        line: usize,
        column: usize,
    },
    /// 管道占位符 `_` 在表达式位置的表示
    Placeholder {
        line: usize,
        column: usize,
    },
}
