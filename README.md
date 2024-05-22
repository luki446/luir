# 🌙 Luir

## 📚 About

Luir is a Lua programming language interpreter written in Rust. It leverages Rust's performance and safety features to provide a robust and efficient interpreter for Lua scripts.

## ✨ Features

- 🦀 **Rust-Powered:** Utilizes Rust for high performance and safety.
- 📜 **Lua Compatibility:** Fully compatible with Lua syntax and semantics.
- 🔗 **Simple Integration:** Easy to integrate into existing Rust/C projects.

## 🛠️ Getting Started

1. Clone the repository:
```bash
git clone https://github.com/luki446/luir
```

2. Build the project:
```bash
cargo build --release
```

3. Run the interpreter on example Lua scripts:
```bash
cargo run --release -- ./example.lua
```

4. Install the interpreter locally as drop-in replacement for `lua`:
```bash
cargo install --path .
```

## 📅 Roadmap to v1.0.0

Release v1.0.0 will mark the first stable complete-ish release of Luir. The following features are planned to be implemented before the release:

- [x] Basic Lua expression parsing and lexing.
- [x] Basic Lua statement parsing and lexing.
- [x] Calling native (Rust implemented) functions from Lua.
- [x] If statement.
- [ ] While loop
- [ ] For loop.
- [ ] Repeat-until loop.
- [ ] Tables and pseudo-OOP.
- [ ] Function definitions.
- [ ] Closures support.
- [ ] More standard Lua library functions.
- [ ] REPL mode.
- [ ] Error handling and reporting. 

## 📝 License

This project is licensed under the BSD 3-Clause License