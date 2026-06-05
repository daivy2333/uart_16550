# tasks.md — 任务追踪

> 由 project-rules-generator 初始化，由 project-docs-assistant 日常维护。
> 2026-06-03: 文档体系迁移到 OpenSpec（任务追踪格式扩展以兼容 OpenSpec changes/）
> 条目格式: <!-- T{编号} --> 标记开头，支持 grep 精确定位。
> OpenSpec 同步: `openspec list` 列出当前 proposals。

---

## 进行中

<!-- 添加时格式: <!-- T{编号} --> - [ ] {任务描述} -->

## 待办

<!-- 添加时格式: <!-- T{编号} --> - [ ] {任务描述} -->

<!-- T1 --> - [x] 同步父 CLAUDE.md 文档索引（uart_16550 行从 .claude/docs/ 改为 openspec/specs/）— 2026-06-05 已完成
<!-- T2 --> - [ ] 评估 optimization O8（DMA 模式寄存器完整控制）在 StarryOS P2 阶段的需求
<!-- T3 --> - [ ] **决策 D1**: 确认 StarryOS Q6 阶段 async_uart wrapper 落地方案（路径 B - StarryOS wrapper 层封装）
<!-- T4 --> - [ ] **决策 D2**: 确认 Waker 实现方式（自研 AtomicWaker vs 引入 embassy 依赖）
<!-- T5 --> - [ ] **决策 D3**: 起草 uart_16550 上游 `embedded-io-async` feature 提案 issue 草稿（暂不提交 PR，先产出文档）

## 阻塞项

<!-- 添加时格式: <!-- T{编号} --> - {阻塞描述} - {原因} -->

## OpenSpec 变更

<!-- 通过 /opsx:propose 创建的变更会出现在 openspec/changes/，归档后用 openspec archive -->
<!-- 当前: 无进行中的 change -->
