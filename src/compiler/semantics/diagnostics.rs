use std::env;

use crate::compiler::ast::SourceSpan;

/// 获取ANSI颜色代码
/// 
/// # 返回值
/// 返回一个元组，包含以下颜色代码：
/// * 粗体红色 (bred) - 用于错误标题
/// * 红色 (red) - 用于错误标记
/// * 粗体蓝色 (bblue) - 用于文件路径
/// * 粗体黄色 (byellow) - 用于建议标题
/// * 暗淡色 (dim) - 用于行号
/// * 重置色 (reset) - 重置所有颜色
/// 
/// # 环境变量支持
/// 如果设置了NO_COLOR环境变量，则返回空字符串（禁用颜色）
#[inline]
pub fn color_codes() -> (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str) {
    // 如果设置了NO_COLOR环境变量则禁用颜色 (https://no-color.org/)
    if env::var("NO_COLOR").is_ok() {
        ("", "", "", "", "", "")
    } else {
        ("\x1b[1;31m", "\x1b[31m", "\x1b[1;34m", "\x1b[1;33m", "\x1b[2m", "\x1b[0m")
    }
}

/// 从源代码中获取指定行的内容
/// 
/// # 参数
/// * `source` - 完整的源代码字符串
/// * `line_no` - 行号（从1开始）
/// 
/// # 返回值
/// * `String` - 指定行的内容，如果行号超出范围则返回空字符串
#[inline]
pub fn get_line(source: &str, line_no: usize) -> String {
    source.lines().nth(line_no - 1).unwrap_or("").to_string()
}

/// 生成错误标记的插入符号字符串
/// 
/// # 参数
/// * `span` - 源码范围，用于确定插入符号的位置和长度
/// 
/// # 返回值
/// * `String` - 插入符号字符串，如 "   ^^^^^"
/// 
/// # 示例
/// 如果span表示第5-10列，则返回 "    ^^^^^^"
#[inline]
pub fn caret_line(span: SourceSpan) -> String {
    let start = span.start.col.saturating_sub(1); // 转换为0基索引
    let width = span.end.col.saturating_sub(span.start.col).max(1); // 确保至少1个字符宽度
    let mut s = String::new();
    
    // 添加前导空格
    for _ in 0..start { 
        s.push(' '); 
    }
    
    // 添加插入符号
    for _ in 0..width { 
        s.push('^'); 
    }
    
    s
}

/// 渲染标准化的Rust风格诊断块（带颜色）
/// 
/// # 参数
/// * `summary` - 错误摘要（第一行，不包含颜色代码）
/// * `filename` - 文件名（显示在头部）
/// * `line_no` - 行号（1基索引）
/// * `col` - 列号（1基索引）
/// * `code_line` - 完整的源码行文本
/// * `caret` - 预构建的插入符号字符串（如 "   ^^^^^"）
/// * `suggestions` - 多行建议文本（已组合好）
/// 
/// # 返回值
/// * `String` - 格式化的错误诊断信息
/// 
/// # 格式示例
/// ```
/// ❌ 错误：你试图修改不可变变量 `x`
///   --> file.kr:3:5
///    |
///  3 | x = x + 1
///    |     ^
/// 💡 修复建议：
///    - 如果你想让它可变，请在首次赋值时加 `$`：
///        $x = 0   ← 这样声明
/// ```
pub fn render_error(summary: &str, filename: &str, line_no: usize, col: usize, code_line: &str, caret: &str, suggestions: &str) -> String {
    let (bred, red, bblue, byellow, dim, reset) = color_codes();
    format!(
        "\n{bred}❌ 错误：{summary}{reset}\n  {bblue}--> {filename}:{line_no}:{col}{reset}\n   |\n {dim}{line_no}{reset} | {line}\n   | {red}{caret}{reset}\n{byellow}💡 修复建议：{reset}\n{suggestions}\n",
        summary = summary,
        filename = filename,
        line_no = line_no,
        col = col,
        line = code_line,
        caret = caret,
        suggestions = suggestions,
        bred = bred,
        red = red,
        bblue = bblue,
        byellow = byellow,
        dim = dim,
        reset = reset,
    )
}
