// SPDX-License-Identifier: Apache-2.0

//! 词法单元定义：`TokenType` 与 `Token`

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // 标识符和字面量
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    TextLiteral(String),
    BoolLiteral(bool),

    // 关键字
    Const,
    True,
    False,
    And,
    Or,
    Not,
    If,
    Else,
    While,
    For,
    In,
    Match,
    Case,
    Break,
    Continue,
    Fun,       // fun
    Return,    // return
    Void,      // Void

    // 运算符
    Assign,          // =
    Plus,            // +
    Minus,           // -
    Multiply,        // *
    Divide,          // /
    Equal,           // ==
    NotEqual,        // !=
    Less,            // <
    Greater,         // >
    LessEqual,       // <=
    GreaterEqual,    // >=
    PlusAssign,      // +=
    MinusAssign,     // -=
    MultiplyAssign,  // *=
    DivideAssign,    // /=
    Increment,       // ++
    Decrement,       // --
    DotDot,          // ..
    DotDotEqual,     // ..=
    PipeArrow,       // |>  管道操作符
    Underscore,      // _  占位符
    Ellipsis,        // ... 可变参数标记
    Dot,             // .  点号

    // 分隔符
    LeftParen,       // (
    RightParen,      // )
    LeftBracket,     // [
    RightBracket,    // ]
    LeftBrace,       // {
    RightBrace,      // }
    Comma,           // ,
    Colon,           // :
    Arrow,           // ->
    Exclamation,     // !

    // 注释
    SingleComment(String),    // # comment
    MultiComment(String),     // --- comment ---
    DocComment(String),       // ** comment

    // 特殊
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}


