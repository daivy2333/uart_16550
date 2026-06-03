# rules.md — 项目编码规范

> 由 project-rules-generator 初始化，由 project-docs-assistant 日常维护。
> 这是三大规则的唯一事实来源。CLAUDE.md 只做索引，不重复规则内容。

---

## Karpathy Guidelines

行为准则，减少 LLM 编码常见错误。

### 1. Think Before Coding

**不假设。不隐藏困惑。暴露权衡。**

实现前：
- 明确陈述假设，不确定就问
- 多种解读存在时，全部呈现 - 不 silently 选择
- 更简单的方法存在时，说出来。必要时 push back
- 不清楚时，STOP。命名困惑点。问。

### 2. Simplicity First

**最小代码解决问题。无投机性功能。**

- 不添加未被要求的功能
- 单次使用代码不抽象
- 未要求的"灵活性"或"可配置性"不加
- 不可能场景的错误处理不加
- 200 行能减到 50 行，重写

### 3. Surgical Changes

**只改必须改。只清理自己的烂摊子。**

- 不"改进"相邻代码、注释、格式
- 不重构没坏的东西
- 匹配现有风格，即使你做法不同
- 删除 YOUR 改动导致的孤儿代码
- 不删除先前存在的死代码（除非被要求）

### 4. Goal-Driven Execution

**定义成功标准。循环直到验证。**

- "添加验证" → "写无效输入测试，然后让它们通过"
- "修复 bug" → "写复现它的测试，然后让它通过"
- "重构 X" → "确保前后测试都通过"

---

## 务实编码原则

整洁代码与务实原则的软件工匠准则。

### 十大铁律

1. **命名即文档** — 精准、可读、可搜索的名称
2. **函数单一职责** — < 20行，只做一件事，无副作用
3. **DRY & 正交性** — 三次法则，模块独立
4. **显式胜于隐式** — 依赖注入，常量命名
5. **健壮边界** — 依赖抽象，核心与框架解耦
6. **可测试设计** — 纯函数优先，依赖可注入
7. **尽早重构** — 小步重构，每次提交更好
8. **务实破窗** — 看到问题立即修，不留给以后
9. **自动化检查** — 格式化、静态分析、测试覆盖
10. **注释解释意图** — 注释"为什么"，不注释"做什么"

---

## Workflow Designer

工作流概念框架，定义执行流程。

### 核心概念

- **Phase** — 逻辑分组的工作容器（进入/退出条件明确）
- **Gate** — 检查点（PASS 或 BLOCK，BLOCK 必须记录原因）
- **Task** — 最小执行单元（可独立验证，完成必须展示证据）
- **Loop** — 重复处理（clarification / review-fix / iteration / retry）

### 执行铁律

1. Phase 进入前必须 Gate PASS
2. Task 开始前必须 Gate PASS
3. Task 完成必须展示证据
4. Loop 退出必须条件 PASS
5. Gate BLOCK 必须记录原因
6. 声明完成必须验证证据

---

## 项目特定规范

### 命名规范

- 类型使用 PascalCase: `Uart16550`, `BaudRate`, `FifoTriggerLevel`
- 函数/方法使用 snake_case: `send_bytes_exact`, `try_receive_byte`
- 常量使用 SCREAMING_SNAKE_CASE: `CLK_FREQUENCY_HZ`, `FIFO_SIZE`, `NUM_REGISTERS`
- bitflags 常量使用 SCREAMING_SNAKE_CASE: `DATA_READY`, `THR_EMPTY`
- 寄存器类型使用全大写: `IER`, `LCR`, `LSR`, `MCR`, `MSR`

### 代码结构

- `#![no_std]` — 必须保持 no_std 兼容
- `#![deny(missing_docs)]` — 所有公开项必须有文档注释
- `#![deny(clippy::all, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]`
- unsafe 块必须有 SAFETY 注释说明安全不变量
- 使用 sealed trait 模式防止外部实现 Backend/RegisterAddress

### 测试规范

- 单元测试在 `#[cfg(test)] mod tests` 中
- 使用 `assert2` 宏进行测试断言
- MMIO 测试使用本地内存数组模拟设备
- API 测试在 `tests/api.rs`

### 提交规范

- 遵循 Conventional Commits: `feat:`, `fix:`, `doc:`, `chore:`, `build(deps):`
- 每次提交前运行: `cargo fmt && cargo clippy && cargo test`

---

## 项目特定约束

### 技术栈

- 语言: Rust (edition 2024, MSRV 1.85.1)
- 框架: no_std 嵌入式库
- 测试: cargo test + assert2
- 格式化: rustfmt + clippy
- 拼写检查: typos (crate-ci/typos)

### 目录结构

- `src/` — 库源码
- `src/backend/` — I/O 后端抽象 (PIO + MMIO)
- `tests/` — 集成测试
- `test/` — 硬件测试子项目（独立 Cargo 项目）

### Git 约束

- 提交信息格式: Conventional Commits
- 分支策略: main + feature 分支 + dependabot 自动合并
- CI: 多平台构建 + spellcheck

---

## Red Flags

❌ 假设不明确 → STOP，问
❌ 过度复杂 → 简化
❌ 改动超出请求 → 回滚
❌ 无测试变更代码 → Iron Law 违规
❌ 顺手添加功能 → Karpathy 违规
❌ Gate BLOCK 不记录 → Workflow 违规
❌ 缺少 SAFETY 注释 → unsafe 块必须注释
❌ 缺少文档注释 → 公开项必须 #[doc]