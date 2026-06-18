# tasks.md — 任务追踪

> 由 project-rules-generator 初始化，由 openspec-assistant 日常维护。
> 2026-06-03: 文档体系迁移到 OpenSpec（任务追踪格式扩展以兼容 OpenSpec changes/）
> 2026-06-17: 同步 feat/uart-16550-async 分支（Q13 async extraction 完成 + 性能优化迭代）
> 条目格式: <!-- T{编号} --> 标记开头，支持 grep 精确定位。
> OpenSpec 同步: `openspec list` 列出当前 proposals。

---

## 进行中

<!-- 添加时格式: <!-- T{编号} --> - [ ] {任务描述} -->

<!-- T7 --> - [ ] feat/uart-16550-async 性能优化迭代：跟踪 overhead（已落地 inline + batch，~43µs/130µs），待 LTO 重新启用评估

## 待办

<!-- 添加时格式: <!-- T{编号} --> - [ ] {任务描述} -->

<!-- T2 --> - [ ] 评估 optimization O8（DMA 模式寄存器完整控制）在 StarryOS P2 阶段的需求
<!-- T5 --> - [ ] **决策 D3**: 起草 uart_16550 上游 `embedded-io-async` feature 提案 issue 草稿（暂不提交 PR，先产出文档）
<!-- T8 --> - [ ] ADR-034 跟踪：LTO 临时禁用，待特性稳定后重新启用（37f60fb）

## 已完成

<!-- 添加时格式: <!-- T{编号} --> - [x] {任务描述} — {完成日期} -->

<!-- T1 --> - [x] 同步父 CLAUDE.md 文档索引（uart_16550 行从 .claude/docs/ 改为 openspec/specs/）— 2026-06-05
<!-- T6 --> - [x] 同步 Q13 async extraction 文档状态（SNAPSHOT.md + tasks.md + references/spec.md 子项目索引）— 2026-06-17
<!-- T3 --> - [x] **决策 D1**: 确认 StarryOS Q6 阶段 async_uart wrapper 落地方案（路径 B - StarryOS wrapper 层封装）— 2026-06-16（Q13 完成，TtyRead/TtyWrite trait 提取）
<!-- T4 --> - [x] **决策 D2**: 确认 Waker 实现方式（自研 AtomicWaker）— 2026-06-16（commit c231ab7，引入 embassy-sync 依赖但 waker 模式自研）
<!-- T10 --> - [x] **Q13 async extraction**: 完整异步 UART 栈提取到 uart_16550（21 commits，async feature gate）— 2026-06-16
<!-- T11 --> - [x] **Q13.1 性能优化**: ring buffer `#[inline(always)]` + `push_batch`/`pop_batch` — 2026-06-16（commits a0cead0 + 73aca5c）
<!-- T12 --> - [x] **Bugfix**: RingBufTx::push() 缺 wake 导致 Shell 挂起 — 2026-06-16（commit de8cd8b）

## 阻塞项

<!-- 添加时格式: <!-- T{编号} --> - {阻塞描述} - {原因} -->

## OpenSpec 变更

<!-- 通过 /opsx:propose 创建的变更会出现在 openspec/changes/，归档后用 openspec archive -->
<!-- 当前: 无进行中的 change（Q13 不回填，Q13 视为已沉淀到 spec/learned/architecture） -->
