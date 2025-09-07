/// 命令行接口模块
mod cli;

/// 编译器模块
mod compiler;

/// Kairo编程语言编译器的主入口点
/// 
/// # 功能
/// 1. 解析命令行参数
/// 2. 执行相应的编译或运行操作
/// 3. 处理错误并显示友好的错误信息
fn main() {
    if let Err(e) = cli::run() {
        // 优先显示根本原因（通常是我们编译器构造的友好消息）
        if let Some(root) = e.chain().last() {
            eprintln!("{}", root);
        } else {
            eprintln!("{}", e);
        }
        std::process::exit(1);
    }
}
