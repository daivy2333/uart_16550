# SNAPSHOT.md - 项目快照

> Generated at 2026-05-25
> Last updated: 2026-05-25

---

## 当前状态

**Phase**: 稳定维护
**Status**: v0.6.0 已发布，当前分支干净，无未提交更改

---

## 项目结构

```
uart_16550/
├── src/
│   ├── lib.rs              # 主入口，Uart16550 类型定义
│   ├── config.rs           # Config、BaudRate 配置类型
│   ├── error.rs            # 错误类型定义
│   ├── spec.rs             # 16550 规范常量、寄存器、计算函数
│   ├── tty.rs              # Uart16550Tty 便捷封装
│   ├── embedded_io.rs      # embedded-io trait 实现
│   └── backend/
│       ├── mod.rs           # Backend trait、RegisterAddress trait
│       ├── mmio.rs          # MMIO 后端实现
│       ├── pio.rs           # x86 Port I/O 后端实现
├── tests/
│   └── api.rs              # API 集成测试
├── test/                    # 硬件测试子项目（独立 Cargo.toml）
├── .github/workflows/      # CI: QA (spellcheck), Build (多平台)
├── Cargo.toml              # 版本 0.6.0, edition 2024
├── .typos.toml             # 拼写检查配置
├── CHANGELOG.md
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
```

---

## 技术栈

| 类别 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust | 1.85.1+ (edition 2024) |
| 核心依赖 | bitflags | 2.11 |
| 可选依赖 | embedded-io | 0.7 |
| 测试 | assert2 | 0.4 |
| CI 拼写检查 | crate-ci/typos | 1.46.0 |
| 本地工具链 | rustc | 1.90.0 |

---

## Git 状态

**当前分支**: dev/optimize
**最近提交**: 55f0b1b - add project documentation system
**未提交更改**: 0 (clean)

---

## 关键文件

| 文件 | 作用 | 行数 | 状态 |
|------|------|------|------|
| src/lib.rs | Uart16550 主类型 + init/send/receive | 1122 | 稳定 |
| src/spec.rs | 寄存器定义 + 常量 + 计算 + 测试 | 1670 | 稳定 |
| src/config.rs | Config/BaudRate 配置类型 | 186 | 稳定 |
| src/tty.rs | Uart16550Tty 便捷封装 | 224 | 稳定 |
| src/error.rs | 错误类型枚举 | 216 | 稳定 |
| src/backend/mod.rs | Backend trait 定义 | 152 | 稳定 |
| src/backend/mmio.rs | MMIO 后端（含 aarch64 特殊处理） | 141 | 稳定 |
| src/backend/pio.rs | x86 Port I/O 后端 | 71 | 稳定 |
| src/embedded_io.rs | embedded-io trait glue | 64 | 稳定 |
| tests/api.rs | API 集成测试 | 26 | 稳定 |

---

## 当前工作

### 进行中

- 优化分支开发（dev/optimize）

### 待办

- 硬件测试子项目 (test/) 可在真实硬件上验证

### 阻塞

- 无阻塞项

---

## 技术决策记录

| 决策 | 原因 | 时间 |
|------|------|------|
| 使用 Backend trait + 泛型 | 统一 PIO/MMIO 访问接口，避免运行时分支 | 项目初始 |
| edition 2024 + resolver 3 | 现代 Cargo 特性，改进依赖解析 | v0.6.0 |
| NonNull<u8> 要求 MMIO 地址 | 防止空指针，类型安全 | 2026-05 |
| aarch64 内联汇编 MMIO | 避免 LLVM 不可虚拟化指令 | 2026-05 |
| sealed trait 模式 | 防止外部实现 Backend/RegisterAddress | 项目初始 |

---

## 最近修改

| 时间 | 文件 | 改动类型 |
|------|------|----------|
| 2026-05 | src/lib.rs, backend/* | NonNull MMIO 地址重构 |
| 2026-05 | tests/api.rs | 添加 API 测试 |
| 2026-05 | CHANGELOG.md | v0.6.0 发布准备 |

---

## 下一步

- 优化分支开发进行中，具体优化方向待定