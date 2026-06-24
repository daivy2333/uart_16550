# ARC-202606241146 归档任务清单

## 1. 准备

- [x] 创建 carrier change 骨架 (mkdir + .openspec.yaml + proposal.md)
- [x] 写入 `specs/optimization/spec.md` (O6 + O9 完整保留)
- [x] 写入 `specs/learned/spec.md` (L_EXP-3 完整保留)
- [x] 写入 `specs/tasks/spec.md` (T31 完整保留)

## 2. 源文档清理

- [ ] optimization/spec.md - 移除 O6 Scenario
- [ ] optimization/spec.md - 移除 O9 Scenario
- [ ] optimization/spec.md - "优化优先级管理" 移除 O9 引用
- [ ] optimization/spec.md - 末尾追加 arc 指引
- [ ] learned/spec.md - 移除"待探索清单"第 3 项
- [ ] learned/spec.md - "依赖关系图" 加 ⚠️ STALE 标记
- [ ] learned/spec.md - 末尾追加 arc 指引
- [ ] tasks.md - 移除 T31 阻塞项
- [ ] tasks.md - 末尾追加 arc 指引
- [ ] tasks.md - 修复 T21 编号冲突 (Q15 M3 → T32)

## 3. CLAUDE.md 同步修复

- [ ] §OS 抽象 Trait 表格 5 → 2 trait + ADR-036 引用

## 4. 验证

- [ ] grep 验证所有墓碑标记
- [ ] git diff 检视变更范围
