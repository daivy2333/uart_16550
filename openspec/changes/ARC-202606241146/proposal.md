# ARC-202606241146 — 项目文档瘦身归档（Q15 完成后）

## Why

Q15 M0~M4 全部完成（commit `0393b22` M3 TtyWrite short-write contract），但文档体系中以下条目已过时：

- **O6 批量读写 API**：commit `73aca5c` 已实现 `push_batch`/`pop_batch`，但 optimization/spec.md 仍标为"待优化"
- **O9 异步串口 ISR + RingBuffer + Waker 集成**：Q15 已在 `feat/uart-16550-async` 分支完成，但 optimization/spec.md 仍标为"高优先级 待启动"
- **L_EXP 待探索第 3 项** "embedded-io feature 在 StarryOS 集成"：已通过 `async` feature 集成（commit `f260fde` `migrate device_ops`）
- **T31 性能退化阻塞项**：用户陈述已完成；lock 竞争已通过 SPSC + batch + inline-always 重构缓解

## 归档条目映射表

| 编号 | 源文档 | 类型 | 原因 | 恢复条件 |
|------|--------|------|------|---------|
| **O6** | optimization/spec.md | Archive | 批量 push/pop 已实现 (73aca5c) | 若将来 ring buffer 需其他批量优化 |
| **O9** | optimization/spec.md | Archive | Q15 M0-M4 全部完成，65 tests GREEN | 若上游拒绝 async feature，需 wrapper 回退 |
| **L_EXP-3** | learned/spec.md | Archive | StarryOS 已用 async feature 集成 | 若需重新评估 embedded-io 集成路径 |
| **T31** | .claude/docs/tasks.md | Archive | 用户陈述已完成 | 若 benchmark 复现 5.4x 开销 |

## 排除项（不动）

- **CLAUDE.md** 内容：仅标记 `💡 SUGGEST-REVIEW`，不归档（规则保护）；用户决定本轮同步修复 §OS 抽象 Trait 表格（5 → 2 trait）
- **A1** (architecture/spec.md)：2026-06-20 活跃 ADR，Keep
- **L_DEP 依赖关系图**：原地 `⚠️ STALE` 标记，不归档（含部分有效信息）
- **O_PRIO 优化优先级列表**：原地 Edit 移除 O9，不归档

## 跨文档交叉引用

| 已归档条目 | 被引用位置 | 处置 |
|------------|-----------|------|
| O9 | optimization/spec.md "优化优先级管理" Requirement 引用 "O9 高优先级" | 原地 Edit 同步移除 O9 |
| T31 | （无） | 无 |
| L_EXP-3 | （无） | 无 |
| O6 | （无） | 无 |

## 恢复协议

用户说"恢复 O6/O9/L_EXP-3/T31"时：

1. `grep -rn "<编号>" openspec/changes/ARC-202606241146/specs/` 定位归档条目
2. 读取 carrier spec 中 `### <编号> (Archive, ...)` 完整内容
3. Edit 复制回原文档原位置
4. 源文档 arc 指引计数 -1，追加 `<!-- restored: <编号> 2026-MM-DD -->`

## 副作用任务（不在 carrier spec 流程内）

1. **L_DEP Stale-Warn**：learned/spec.md `### Requirement: 依赖关系图` 末尾加 `⚠️ STALE` 标记
2. **O_PRIO 同步更新**：optimization/spec.md "优化优先级管理" Scenario 移除 O9
3. **CLAUDE_OS 同步修复**：CLAUDE.md §OS 抽象 Trait 表格 5 个 trait → 2 个 trait + ADR-036 引用
4. **T21 编号冲突修复**：tasks.md 中 `<!-- T21 -->` Q15 M3 重新编号为 `<!-- T32 -->`
