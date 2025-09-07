use std::{path::PathBuf, process::Command};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use crate::compiler;

/// Kairo命令行接口 - 运行和构建.kr文件
/// 
/// 使用clap库提供现代化的命令行参数解析
#[derive(Parser, Debug)]
#[command(name = "kairo", version, about = "Kairo language toolchain (MVP)")]
struct Cli {
    /// 子命令
    #[command(subcommand)]
    command: Commands,
}

/// 支持的命令类型
#[derive(Subcommand, Debug)]
enum Commands {
    /// 直接运行.kr文件（编译为临时可执行文件然后执行）
    Run {
        /// .kr源文件路径
        file: PathBuf,
    },
    /// 将.kr文件构建为可执行文件
    Build {
        /// .kr源文件路径
        file: PathBuf,
        /// 使用优化构建
        #[arg(long)]
        release: bool,
    },
}

/// 运行CLI程序
/// 
/// # 返回值
/// * `Result<()>` - 成功返回Ok(())，失败返回错误信息
/// 
/// # 功能
/// 1. 解析命令行参数
/// 2. 根据子命令执行相应操作
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { file } => run_file(file),
        Commands::Build { file, release } => build_file(file, release).map(|_| ()),
    }
}

/// 运行.kr文件
/// 
/// # 参数
/// * `file` - .kr源文件路径
/// 
/// # 返回值
/// * `Result<()>` - 成功返回Ok(())，失败返回错误信息
/// 
/// # 功能
/// 1. 验证文件扩展名
/// 2. 编译为可执行文件
/// 3. 执行编译后的程序
fn run_file(file: PathBuf) -> Result<()> {
    ensure_kr_ext(&file)?;

    // 编译为可执行文件（调试模式）
    let exe_path = compiler::compile_file_to_exe(&file, /*release=*/ false)
        .with_context(|| format!("failed to compile {:?}", file))?;

    // 执行编译后的二进制文件
    let status = Command::new(&exe_path)
        .status()
        .with_context(|| format!("failed to run {:?}", exe_path))?;

    if !status.success() {
        return Err(anyhow!("program exited with status: {}", status));
    }
    Ok(())
}

/// 构建.kr文件为可执行文件
/// 
/// # 参数
/// * `file` - .kr源文件路径
/// * `release` - 是否使用发布模式（优化）
/// 
/// # 返回值
/// * `Result<PathBuf>` - 成功返回可执行文件路径，失败返回错误信息
/// 
/// # 功能
/// 1. 验证文件扩展名
/// 2. 编译为可执行文件
/// 3. 显示输出路径
fn build_file(file: PathBuf, release: bool) -> Result<PathBuf> {
    ensure_kr_ext(&file)?;

    // 编译为可执行文件
    let exe_path = compiler::compile_file_to_exe(&file, release)
        .with_context(|| format!("failed to compile {:?}", file))?;

    // 为用户方便显示输出路径
    println!("Built: {}", exe_path.display());
    Ok(exe_path)
}

/// 确保文件具有.kr扩展名
/// 
/// # 参数
/// * `path` - 要检查的文件路径
/// 
/// # 返回值
/// * `Result<()>` - 如果文件存在且具有.kr扩展名返回Ok(())，否则返回错误
/// 
/// # 检查项目
/// 1. 文件是否存在
/// 2. 文件扩展名是否为.kr
fn ensure_kr_ext(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("source file not found: {}", path.display()));
    }
    if path.extension().and_then(|s| s.to_str()) != Some("kr") {
        return Err(anyhow!("expect a .kr file: {}", path.display()));
    }
    Ok(())
}
