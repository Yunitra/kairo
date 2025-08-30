// SPDX-License-Identifier: Apache-2.0

//! AST 公共结构定义

use crate::types::{KairoType, KairoValue};

use super::expression::Expression;

#[derive(Debug, Clone)]
pub struct Block {
    /// 代码块内部的语句序列
    pub statements: Vec<super::statement::Statement>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    /// 模式
    pub pattern: Pattern,
    /// 守卫表达式（可选）
    pub guard: Option<Expression>,
    /// 命中分支后执行的代码块
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct MatchArmExpr {
    /// 模式
    pub pattern: Pattern,
    /// 守卫表达式（可选）
    pub guard: Option<Expression>,
    /// 表达式形式 match 的分支值
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    /// 字面量匹配
    Literal(KairoValue),
    /// 标识符匹配（并进行绑定）
    Identifier(String),
    /// 通配符 `_`
    Wildcard,
    /// 区间匹配（仅整数），inclusive 表示是否包含上界
    Range { start: KairoValue, end: KairoValue, inclusive: bool },
    /// in 表达式匹配，例如 `in 1..3`
    In(Expression),
    /// 元组结构匹配
    Tuple(Vec<Pattern>),
    /// 类型匹配并绑定变量，如 `x: Int`
    TypePattern { var: String, type_name: String },
}

#[derive(Debug, Clone)]
pub struct Parameter {
    /// 参数名称
    pub name: String,
    /// 参数是否可变
    pub mutable: bool,
    /// 参数类型
    pub param_type: KairoType,
    /// 默认值（可选）
    pub default_value: Option<Expression>,
    /// 是否为可变参数（...）
    pub variadic: bool,
}

#[derive(Debug, Clone)]
pub struct Program {
    /// 程序的顶层语句列表
    pub statements: Vec<super::statement::Statement>,
}
