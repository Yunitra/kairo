/// 源代码中的位置信息
/// 用于表示源码中的行号和列号，用于错误报告和调试
#[derive(Debug, Clone, Copy)]
pub struct SourcePos {
    /// 行号（从1开始）
    pub line: usize,
    /// 列号（从1开始）
    pub col: usize,
}

/// 源代码中的范围信息
/// 表示从start位置到end位置的一段源码范围
/// 用于标记语法错误、变量声明位置等
#[derive(Debug, Clone, Copy)]
pub struct SourceSpan {
    /// 起始位置
    pub start: SourcePos,
    /// 结束位置
    pub end: SourcePos,
}

impl SourceSpan {
    /// 创建一个单行的源码范围
    /// 
    /// # 参数
    /// * `line` - 行号（从1开始）
    /// * `start_col` - 起始列号（从1开始）
    /// * `end_col` - 结束列号（从1开始）
    /// 
    /// # 返回值
    /// 返回表示单行范围的SourceSpan
    pub fn single_line(line: usize, start_col: usize, end_col: usize) -> Self {
        Self {
            start: SourcePos { line, col: start_col },
            end: SourcePos { line, col: end_col },
        }
    }
}
