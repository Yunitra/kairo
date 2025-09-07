# 🌅 Kairo 编程语言 —— 为非专业者打造的现代安全高性能语言

> **“像 Python 一样简单，像 Rust 一样快，像 Swift 一样安全 —— 无需学习高阶概念。”**

![Rust](https://img.shields.io/badge/Rust-2024-black?style=for-the-badge&logo=rust)
![Version](https://img.shields.io/badge/Version-0.1.0-blue?style=for-the-badge)
![License](https://img.shields.io/badge/License-Apache%202.0-green?style=for-the-badge)

---

## 📌 一句话使命

> Kairo 是一门为**非计算机专业者、编程新手、领域专家（如科学家、金融分析师、设计师）** 设计的通用编程语言，目标是让他们能**像写 Python 一样轻松地编写代码，同时默认获得接近 Rust 或 C 的性能与内存/类型安全，而无需理解并发、生命周期、指针或类型注解等复杂概念。**

---

## 🎯 核心设计原则

### 1. **渐进式严谨性（Progressive Rigor）**
- 新手第一行代码：`print("Hello")` —— 和 Python 一样简单。
- 当需要状态修改时，引入 `$` 标记可变性：`$count = 0` —— 渐进式学习，无负担。
- 错误处理用 `try` / `must` —— 不强迫理解 `Result<T,E>`，但保证安全。

### 2. **零配置高性能（Zero-Config Performance）**
- 自动并行化：简单循环、数组操作，编译器自动多线程。
- 默认栈分配 + ARC 内存管理 —— 无 GC 停顿，无手动内存管理。
- 静态类型推断 —— 无需写类型，但编译器知道一切，生成最优机器码。

### 3. **默认安全（Safety by Default）**
- 默认不可变变量 —— 避免意外状态变更。
- 可变性必须显式声明（`$`）—— 编译器可做逃逸分析、别名分析。
- 类型安全、内存安全、无空指针、无数组越界（运行时 panic 或编译时检查）。
- 并发安全：无共享状态，通信只通过 `channel`。

### 4. **无高阶概念负担**
- 无生命周期、无借用检查、无 trait、无泛型语法（对用户透明）。
- 所有复杂性由编译器在后台处理，用户只面对直观语法。

---

## 🧩 语言特性速览

### 🔤 语法风格
- 类 Python 基础 + 自创增强。
- 使用 `fun` 关键字定义函数，大括号 `{}` 作用域（非缩进）。
- 可变变量用 `$` 前缀**声明**（仅在定义时）：`$count = 0`，后续使用无需 `$`。
- 不可变变量默认：`name = "Alice"` —— 重赋值报错。

### 🧮 类型系统
- **静态类型推断** —— 用户可不写类型，编译器自动推导。
- 支持显式类型注解（可选）：`fun add(a: int, b: int) -> int { ... }`
- 无隐式类型转换 —— `1 + 2.0` 报错，需显式 `float(1) + 2.0`。

### 🧵 并发模型
- **自动并行化**：编译器自动并行化纯函数循环。
- **显式并发**：`spawn` + `await` + `channel` —— 像调函数一样简单。
  ```rust
  $task = spawn download(url)
  result = await task
  ```
- 无数据竞争：编译器静态检查共享可变状态。

### 🚨 错误处理
- `try`：尝试执行，失败则提前返回错误。
- `must`：强制成功，失败则 panic（带完整调用栈和修复建议）。
- 所有函数默认返回 `Result<T, E>`，但语法隐藏 —— 用户只写 `return value` 或 `return error("msg")`。

### 🧠 内存管理
- ARC（自动引用计数） + 栈分配优化。
- 小数据（int, 小字符串）栈分配，零开销。
- 大数据堆分配 + ARC，无 GC 停顿。
- 循环引用检测 → 编译器报错 + 建议用 weak。

### 📦 标准库（Batteries Included）
- fs：文件读写。
- http：网络请求（GET/POST/JSON）。
- str / regex：字符串与正则。
- list / dict / set / queue：数据结构。
- math / random：数学与随机。
- time / datetime：时间处理。
- test：内置测试框架。
- cli：命令行参数解析。

### 🛠️ 工具链
- 编译器：友好错误信息 + 修复建议。
- REPL：智能补全 + 文档悬浮 + 历史搜索。
- 调试器：可视化变量 + 时间旅行调试（可选）。
- IDE 插件（VS Code / JetBrains）：实时错误 + 补全 + 跳转定义。
- 文档生成器：注释 → HTML，示例即测试（DocTest）。

### ⚙️ 运行时与部署
- 开发时：混合模式（AOT + JIT 缓存）→ 改代码 → 秒级重跑。
- 发布时：编译成本地机器码（Windows/Mac/Linux）→ 双击即用，无依赖。
- 可选：编译成 WebAssembly → 浏览器运行。

### 🏗️ 实现技术栈
- 实现语言：Rust（从头到尾统一语言）。
- 目标后端：
- 第一阶段：生成 Rust 代码（利用 `Rc<RefCell<T>>` 实现 ARC）。
- 后期可选：Cranelift / LLVM。
- 构建工具：Cargo。
- 词法/语法分析：logos + lalrpop（后续引入）。

## 📈 项目路线图（Roadmap）

### 🚶 第 1 个月：最小可用编译器（MVP）

- 支持 `print("...")`
- 支持变量声明 `x = 10` 和 `$count = 0`
- 生成 Rust 代码 → 调用 rustc → 运行

### 🏗️ 第 3 个月：核心语言特性

- 函数 `fun main() { ... }`
- `for` 循环 + 自动并行化
- `if` 条件
- `try` / `must` 错误处理
- 基础类型推断

### 🚀 第 6 个月：标准库 + 工具链
- `fs`, `http`, `str` 等模块
- REPL 交互环境
- VS Code 插件（基础补全 + 错误提示）

### 🌟 第 12 个月：v1.0 发布
- 完整标准库
- 调试器 + 文档生成
- 多平台编译（Windows/Mac/Linux）
- 性能优化（切换到 Cranelift/LLVM 后端）
- 官网 + 教程 + 10+ 示例项目

## 🤝 贡献指南

> 欢迎任何人贡献！无论你是新手还是专家。 

- 新手友好任务：标有 good first issue。
- 文档贡献：教程、示例、注释。
- 标准库扩展：新增模块（如 json, csv）。
- 工具链增强：REPL、调试器、IDE 插件。
- 性能优化：后端切换、并行化增强。

## 📄 许可证

本项目采用 Apache-2.0 许可证 - 查看 [LICENSE](.\LICENSE) 文件了解详情。

## 💬 联系我们
GitHub: `https://github.com/Yunitra/kairo`
<!--Discord: `#kairo-lang` (待创建)-->
<!--邮箱: `team@kairo-lang.dev` (待创建)-->

> Kairo 不是“另一个编程语言” —— 它是为那些“不想成为程序员，但需要编程解决问题”的人而生的工具。 

让我们一起，让编程回归“表达思想”，而非“讨好机器”。