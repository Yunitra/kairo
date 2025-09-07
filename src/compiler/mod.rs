/// 代码生成模块
/// 将AST转换为目标语言代码
pub mod codegen;

use std::{fs, path::{Path, PathBuf}, process::Command};

use anyhow::{Context, Result};

/// 解析器模块
/// 将源代码解析为抽象语法树
#[path = "parser/mod.rs"]
pub mod parser;

/// 语义分析模块
/// 执行类型检查、变量声明验证等语义分析
#[path = "semantics/mod.rs"]
pub mod semantics;

/// 抽象语法树模块
/// 定义Kairo语言的AST节点类型
#[path = "ast/mod.rs"]
pub mod ast;

use semantics::check_semantics;

/// 将.kr源文件编译为可执行文件（Windows上为.exe）
/// 
/// # 参数
/// * `src_path` - 源文件路径
/// * `release` - 是否使用发布模式（优化）
/// 
/// # 返回值
/// * `Result<PathBuf>` - 成功返回可执行文件路径，失败返回错误信息
/// 
/// # 编译流程
/// 1. 读取源文件
/// 2. 解析为抽象语法树
/// 3. 执行语义分析
/// 4. 生成Rust代码
/// 5. 调用rustc编译为可执行文件
pub fn compile_file_to_exe(src_path: &Path, release: bool) -> Result<PathBuf> {
    // 读取源文件内容
    let source = fs::read_to_string(src_path)
        .with_context(|| format!("failed to read source: {}", src_path.display()))?;

    // 解析为抽象语法树
    let program = parser::parse(&source, src_path)?;
    
    // 执行语义分析
    let semantic = check_semantics(&program, src_path, &source)?;

    // 生成Rust代码
    let rust_code = codegen::rust::generate_rust(&program, &semantic)?;

    // 准备输出路径
    let file_stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out");

    let out_dir = PathBuf::from("target").join("kairo_out");
    fs::create_dir_all(&out_dir).with_context(|| format!("create dir: {}", out_dir.display()))?;

    let rs_path = out_dir.join(format!("{file_stem}.rs"));
    let exe_name = if cfg!(target_os = "windows") { 
        format!("{file_stem}.exe") 
    } else { 
        file_stem.to_string() 
    };
    let exe_path = out_dir.join(exe_name);

    // 写入生成的Rust代码
    fs::write(&rs_path, rust_code).with_context(|| format!("write file: {}", rs_path.display()))?;

    // 调用rustc编译
    let mut cmd = Command::new("rustc");
    if release {
        cmd.arg("-O"); // 优化标志
    }
    let status = cmd
        .arg("--edition=2024")
        .arg("-o")
        .arg(&exe_path)
        .arg(&rs_path)
        .status()
        .with_context(|| format!("failed to run rustc for {}", rs_path.display()))?;

    if !status.success() {
        anyhow::bail!("rustc failed to compile generated code. See above errors.");
    }

    Ok(exe_path)
}
