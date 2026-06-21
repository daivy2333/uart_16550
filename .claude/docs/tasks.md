# tasks.md — 任务追踪

> 由 openspec-assistant 日常维护。
> Last updated: 2026-06-21 (M4 Sync 已回退，Q15 增量重融合)
> 2026-06-21: M4 Sync 回退到 pre-M4 基线（60c5729），原代码保留在 feat/uart-16550-async-temp。
> Q15 阶段从 pre-M4 基线出发，按最小可验证单元重新 apply M4+ 正确性修复。
> 条目格式: <!-- T{编号} --> 标记开头，支持 grep 精确定位。

---

## 进行中

（无）

## 待办

<!-- T2 --> - [ ] 评估 optimization O8（DMA 模式寄存器完整控制）在 StarryOS P2 阶段的需求
<!-- T5 --> - [ ] **决策 D3**: 起草 uart_16550 上游 `embedded-io-async` feature 提案 issue 草稿（暂不提交 PR，先产出文档）
<!-- T8 --> - [ ] ADR-034 跟踪：LTO 临时禁用，待特性稳定后重新启用

## 已完成

<!-- T13 --> - [x] **M4.1 ring 指标**: RingBufRx/Tx 指标 — 2026-06-19
<!-- T14 --> - [x] **M4.2 copier 指标**: AsyncUartDriver 指标 — 2026-06-19
<!-- T15 --> - [x] **M4.3 waker 顺序修复**: register→enable 顺序 — 2026-06-19
<!-- T16 --> - [x] **M4.4 TX backpressure**: busy-poll 修复 — 2026-06-19
<!-- T17 --> - [x] **M4.5 TDD 测试**: 4 RED→GREEN 测试 — 2026-06-19
<!-- T18 --> - [x] **M4.6 全量测试**: 58 tests GREEN — 2026-06-19
<!-- T19 --> - [x] **正确性修复**: F1-F12 全部修复，12 commits，65 tests GREEN，clippy clean — 2026-06-20
<!-- T20 --> - [x] **F1**: SPSC 别名 UB → RawMutex + take_reader gate + copier gate
<!-- T21 --> - [x] **F2**: async I/O 不等待 → register→recheck→Pending + flush TEMT
<!-- T22 --> - [x] **F3**: TX 判空丢唤醒 → TX copier register→recheck 协议
<!-- T23 --> - [x] **F4**: 全局 waker → per-driver AtomicWakers + handle_irq
<!-- T24 --> - [x] **F5**: ISR bypass backend → UartPort IRQ 方法
<!-- T25 --> - [x] **F6**: IER RMW → update_ier 在 SpinNoIrq 锁内
<!-- T26 --> - [x] **F7**: 中断源不清 → Line/Modem/DMA 全部处理
<!-- T27 --> - [x] **F9**: loopback 不恢复 → finally-style MCR/IER/FCR 恢复
<!-- T28 --> - [x] **F10**: NAPI 无预算 → 4096-byte budget + yield
<!-- T29 --> - [x] **F11**: 计算 panic → checked arithmetic + Error
<!-- T30 --> - [x] **F12**: 测试不真实 → 每测试独立 storage + poll 路径

## 阻塞项

<!-- T31 --> - **⚠️ 性能退化**: write+tcdrain benchmark 5.4x 开销。RingBufTx::push() 每调用获取 SpinNoIrq + RefCell::borrow_mut，ISR handle_irq 路径同样获取 SpinNoIrq（ArceOsUartPort 方法）。两路径竞争同一 SpinNoIrq 实例，形成锁竞争退化。待优化方向：(1) Mutex 内 RefCell→UnsafeCell (2) handle_irq 单次锁复用。

## OpenSpec 变更

- `fix-uart-correctness-invariants` — 12 项修复全部完成，65 tests GREEN，待归档
