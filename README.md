# Kairo 编程语言解释器

![Rust](https://img.shields.io/badge/Rust-2024-black?style=for-the-badge&logo=rust)
![Version](https://img.shields.io/badge/Version-0.1.0-blue?style=for-the-badge)
![License](https://img.shields.io/badge/License-Apache%202.0-green?style=for-the-badge)

Kairo 是一门现代化的、简洁优雅的编程语言，由 Rust 实现的全功能解释器。本项目采用高度模块化的架构设计，具有出色的性能和开发体验。

## ✨ 特性亮点

- 🚀 **高性能**：基于 Rust 实现，内存安全，执行高效
- 🎯 **现代化语法**：简洁直观的语法设计，学习曲线平缓
- 🔧 **功能丰富**：支持函数式编程、面向对象特性、模式匹配等
- 📦 **模块化架构**：深度模块化设计，便于维护和扩展
- 🛠️ **开发友好**：完善的错误提示和调试信息
- 📚 **中文支持**：全面的中文文档和错误信息

## 📦 安装

### 方式一：从源码编译（推荐）

```bash
# 克隆仓库
git clone https://github.com/Yunitra/kairo.git
cd Kairo

# 编译
cargo build --release

# 安装到系统（可选）
cargo install --path .
```

### 方式二：使用预编译二进制

访问 [Releases](https://github.com/Yunitra/kairo/releases) 页面下载最新版本。

## 🚀 快速开始

### 创建第一个程序

创建文件 `hello.kai`：

```kairo
# 第一个 Kairo 程序
name = "Kairo"
print("Hello,", name, "!")
```

### 运行程序

```bash
# 方式1：直接运行
kairo hello.kai

# 方式2：编译后运行
cargo run hello.kai

# 方式3：拖拽文件到可执行文件上（Windows）
# 直接将 .kai 文件拖拽到 kairo.exe 上
```

## 📖 语言特性

### 基本类型

Kairo 支持多种基础数据类型：

```kairo
# 整数和浮点数
age = 25
height = 175.5

# 字符串
name = "Alice"
message = 'Hello World'

# 布尔值
is_active = true
has_permission = false
```

### 变量和常量

```kairo
# 可变变量
x! = 10
x = x + 5

# 不可变变量
y = 20

# 常量
const PI = 3.14159
const APP_NAME = "My App"
```

### 函数定义

```kairo
# 单行函数
fun add(x: Int, y: Int) -> Int = x + y

# 多行函数
fun factorial(n: Int) -> Int {
  if n <= 1 {
    return 1
  } else {
    return n * factorial(n - 1)
  }
}

# 默认参数
fun greet(name: Text, prefix: Text = "Hello") -> Text = prefix + ", " + name

# 可变参数
fun sum(nums: Int...) -> Int {
  result! : Int = 0
  for num in nums {
    result += num
  }
  return result
}
```

### 集合类型

```kairo
# 列表
numbers = [1, 2, 3, 4, 5]
fruits = ["apple", "banana", "cherry"]

# 元组
point = (10, 20)
person = ("Alice", 30, true)

# 映射
user = {"name": "Bob", "age": 25, "active": true}
scores = {"math": 95, "english": 87}
```

### 控制流

```kairo
# if/else 语句
age = 25
if age < 18 {
  print("未成年")
} else if age < 65 {
  print("成年")
} else {
  print("老年人")
}

# while 循环
i! = 0
while i < 5 {
  print("计数:", i)
  i += 1
}

# for 循环
numbers = [1, 2, 3, 4, 5]
for num in numbers {
  print("数字:", num)
}

# 遍历映射
scores = {"Alice": 95, "Bob": 87}
for name, score in scores {
  print(name, "的分数:", score)
}
```

### 模式匹配

```kairo
# 值匹配
value = 2
match value {
  1 -> { print("一") }
  2 -> { print("二") }
  3 -> { print("三") }
  _ -> { print("其他") }
}

# 范围匹配
score = 85
match score {
  in 0..60 -> { print("不及格") }
  in 60..80 -> { print("及格") }
  in 80..=100 -> { print("优秀") }
  _ -> { print("无效分数") }
}

# 元组匹配
point = (5, 10)
match point {
  (0, 0) -> { print("原点") }
  (x, 0) -> { print("x轴") }
  (0, y) -> { print("y轴") }
  (x, y) -> { print("坐标:", x, y) }
}
```

### 扩展函数

```kairo
# 为 List 类型添加方法
fun List[T].length() -> Int {
  count! : Int = 0
  for item in this {
    count += 1
  }
  return count
}

# 为 Map 类型添加方法
fun Map[K->V].keys() -> List[K] {
  result! : List[K] = []
  for key, value in this {
    result = result + [key]
  }
  return result
}

# 使用扩展函数
my_list = [1, 2, 3, 4, 5]
print("长度:", my_list.length())

my_map = {"a": 1, "b": 2, "c": 3}
print("键列表:", my_map.keys())
```

### 管道操作符

```kairo
# 基本管道
result = 5 |> add(_, 10)

# 链式管道
result = 2 |> add(_, 3) |> factorial

# 实际示例
numbers = [1, 2, 3, 4, 5]
sum = numbers |> filter(_, is_even) |> reduce(_, add, 0)
```

## 🏗️ 项目结构

```
kairo/
├── src/
│   ├── main.rs             # CLI 入口
│   ├── lib.rs              # 库入口，导出所有模块
│   ├── lexer/              # 词法分析器
│   │   ├── mod.rs          # 词法分析器入口
│   │   ├── token.rs        # Token 定义
│   │   ├── reader.rs       # 字符读取
│   │   ├── string.rs       # 字符串处理
│   │   ├── number.rs       # 数字解析
│   │   ├── identifier.rs   # 标识符处理
│   │   ├── comment.rs      # 注释处理
│   │   └── scanner.rs      # 主扫描逻辑
│   ├── parser/             # 语法分析器
│   │   ├── mod.rs          # 语法分析器入口
│   │   ├── expressions.rs  # 表达式解析
│   │   ├── statements.rs   # 语句解析
│   │   ├── pattern.rs      # 模式解析
│   │   ├── types.rs        # 类型解析
│   │   ├── assignments.rs  # 赋值解析
│   │   └── functions.rs    # 函数解析
│   ├── ast/                # 抽象语法树
│   │   ├── mod.rs          # AST 入口
│   │   ├── expression.rs   # 表达式节点
│   │   ├── statement.rs    # 语句节点
│   │   └── common.rs       # 通用节点
│   ├── interpreter/        # 解释器
│   │   ├── mod.rs          # 解释器入口
│   │   ├── exec.rs         # 语句执行
│   │   ├── eval.rs         # 表达式求值
│   │   ├── scope.rs        # 作用域管理
│   │   ├── function.rs     # 函数处理
│   │   ├── pattern.rs      # 模式匹配
│   │   └── control_flow.rs # 控制流
│   ├── types/              # 类型系统
│   │   ├── mod.rs          # 类型系统入口
│   │   ├── type.rs         # 类型定义
│   │   └── value.rs        # 值定义
│   └── error.rs            # 错误处理
├── Cargo.toml              # 项目配置
└── README.md               # 项目说明
```

## 🛠️ 开发指南

### 环境要求

- Rust 1.70+
- Cargo 包管理器

### 构建和测试

```bash
# 开发构建
cargo build

# 运行测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

### 代码规范

- 使用 `cargo fmt` 保持一致的代码格式
- 使用 `cargo clippy` 检查代码质量
- 遵循 Rust 官方编码规范
- 提交前务必运行 `cargo fmt && cargo clippy && cargo build`

## 🤝 贡献指南

我们欢迎各种形式的贡献！

### 贡献步骤

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 贡献类型

- 🐛 **Bug 修复**
- ✨ **新功能**
- 🛠️ **代码重构**

## 📄 许可证

本项目采用 Apache-2.0 许可证 - 查看 [LICENSE](.\LICENSE) 文件了解详情。

## 🙋‍♂️ 联系我们

- 项目主页：[GitHub](https://github.com/Yunitra/kairo)
- 问题反馈：[Issues](https://github.com/Yunitra/kairo/issues)
- 讨论交流：[Discussions](https://github.com/Yunitra/kairo/discussions)

<p align="center">
  <strong>用 ❤️ 和 Rust 打造的现代化编程语言</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/github/stars/Yunitra/kairo?style=social" alt="GitHub stars">
  <img src="https://img.shields.io/github/forks/Yunitra/kairo?style=social" alt="GitHub forks">
</p>
