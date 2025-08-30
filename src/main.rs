// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use colored::*;
use std::fs;
use std::path::Path;
use std::io;

use kairo::{Lexer, Parser as KairoParser, Interpreter};

#[derive(Parser)]
#[command(name = "kairo")]
#[command(about = "Kairo编程语言解释器")]
#[command(version = "0.1.0")]
struct Cli {
    /// .kai源文件路径
    #[arg(help = "要执行的.kai文件")]
    file: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match run(cli) {
        Ok(_) => {
            // 保持命令行窗口打开
            println!("\n{}", "程序执行完成。按任意键退出...".green());
            let _ = io::stdin().read_line(&mut String::new());
        }
        Err(e) => {
            eprintln!("{}", e);
            println!("\n{}", "程序执行失败。按任意键退出...".red());
            let _ = io::stdin().read_line(&mut String::new());
            std::process::exit(1);
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = match cli.file {
        Some(path) => path,
        None => {
            eprintln!("{}", "错误: 请提供.kai文件路径".red());
            eprintln!("用法: kairo <file.kai>");
            eprintln!("或者直接将.kai文件拖拽到kairo.exe上");
            return Ok(());
        }
    };

    // 检查文件是否存在
    if !Path::new(&file_path).exists() {
        return Err(format!("文件不存在: {}", file_path).into());
    }

    // 检查文件扩展名
    if !file_path.ends_with(".kai") {
        return Err("只能执行.kai文件".into());
    }

    println!("{} {}", "正在执行:".blue(), file_path.yellow());
    println!("{}", "=".repeat(50).blue());

    // 读取源代码
    let source = fs::read_to_string(&file_path)?;

    // 词法分析
    let mut lexer = Lexer::new(source.clone());
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            let line = e.line;
            let e = e.with_source_line(get_source_line(&source, line));
            return Err(Box::new(e));
        }
    };

    // 语法分析
    let mut parser = KairoParser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            let line = e.line;
            let e = e.with_source_line(get_source_line(&source, line));
            return Err(Box::new(e));
        }
    };

    // 解释执行
    let mut interpreter = Interpreter::new();
    match interpreter.interpret(ast) {
        Ok(_) => {}
        Err(e) => {
            let line = e.line;
            let e = e.with_source_line(get_source_line(&source, line));
            return Err(Box::new(e));
        }
    }

    Ok(())
}

fn get_source_line(source: &str, line_number: usize) -> String {
    source
        .lines()
        .nth(line_number.saturating_sub(1))
        .unwrap_or("")
        .to_string()
}
