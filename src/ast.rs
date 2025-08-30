// SPDX-License-Identifier: Apache-2.0

//! AST 模块入口：拆分为表达式、语句与通用结构三部分
//!
//! - `common.rs`：公共结构（Block/Pattern/MatchArm/Parameter/Program 等）
//! - `expression.rs`：表达式与相关运算符
//! - `statement.rs`：语句节点

mod common;
pub(crate) mod expression;
pub(crate) mod statement;

pub use common::{Block, MatchArm, MatchArmExpr, Parameter, Pattern, Program};
pub use expression::{AssignmentOperator, BinaryOperator, Expression, UnaryOperator};
pub use statement::{Statement, StatementKind};
