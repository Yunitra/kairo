// SPDX-License-Identifier: Apache-2.0

//! 语句节点定义

use crate::types::KairoType;

use super::common::{Block, MatchArm, Parameter, ErrorField};
use super::expression::Expression;

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    /// 变量或常量声明
    VariableDeclaration {
        name: String,
        mutable: bool,
        explicit_type: Option<KairoType>,
        value: Expression,
        is_const: bool,
    },

    /// 普通函数声明
    FunctionDeclaration {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<KairoType>,
        /// 块函数体（可选，单表达式函数则为 None）
        body: Option<Block>,
        /// 单表达式函数体（可选）
        body_expr: Option<Expression>,
        /// 函数可能抛出的错误类型
        raises: Option<Vec<String>>,
    },

    /// 扩展函数声明（Type.method）
    ExtensionFunction {
        type_name: String,
        method_name: String,
        parameters: Vec<Parameter>,
        return_type: Option<KairoType>,
        body: Option<Block>,
        body_expr: Option<Expression>,
        /// 函数可能抛出的错误类型
        raises: Option<Vec<String>>,
    },

    /// 返回语句
    Return { value: Option<Expression> },

    /// 表达式语句
    Expression(Expression),

    /// if/else 语句
    If {
        condition: Expression,
        then_branch: Block,
        else_ifs: Vec<(Expression, Block)>,
        else_branch: Option<Block>,
    },

    /// while 语句
    While { condition: Expression, body: Block },

    /// for 语句
    For {
        variable: String,
        value_variable: Option<String>,
        iterable: Expression,
        body: Block,
    },

    /// match 语句
    Match { value: Expression, arms: Vec<MatchArm> },

    /// break 语句（可带层数）
    Break { levels: Option<usize> },

    /// continue 语句
    Continue,

    /// 独立代码块
    Block(Block),

    /// 错误定义语句
    ErrorDefinition {
        name: String,
        /// 可选的错误数据字段
        fields: Option<Vec<ErrorField>>,
        /// 错误组定义（例如：err IOErrors = FileNotFound, NetworkTimeout）
        error_group: Option<Vec<String>>,
    },

    /// fail 语句（抛出错误）
    Fail {
        error_name: String,
        /// 错误数据（可选）
        data: Option<Vec<(String, Expression)>>,
    },
}
