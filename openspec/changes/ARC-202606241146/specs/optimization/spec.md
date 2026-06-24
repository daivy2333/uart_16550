# optimization 域归档条目

> Carrier spec for ARC-202606241146
> 源文档: openspec/specs/optimization/spec.md

---

## 完整保留区

### O6 (Archive, 2026-06-24) — 批量读写 API

**原始 Requirement**（位于源文档"已识别优化点"区域）：

- **WHEN** copier 任务逐字节调用 `try_receive`/`try_send`
- **THEN** 当前影响：每次都读 LSR 寄存器，寄存器访问开销累积
- **AND** 建议方案：添加批量版本，一次 LSR 检查后连续读/写多字节，减少 MMIO 访问次数
- **AND** 优先级：中 | 触发条件：copier 循环成为性能瓶颈时

**归档原因**：commit `73aca5c` `perf(uart-async): add batch push/pop to reduce lock overhead` 已实现 `push_batch`/`pop_batch` API。

**验证**：
- `src/async_/ring_buffer.rs:95` `pub fn push_batch(&self, data: &[u8]) -> usize`
- `src/async_/ring_buffer.rs:215` `pub fn pop_batch(&self, buf: &mut [u8]) -> usize`
- `src/async_/driver.rs:236` `self.rx.push_batch(&read_buf[..total])`
- `src/async_/driver.rs:284` `pending = self.tx.pop_batch(&mut write_buf)`

**恢复条件**：若未来需要更激进的批量策略（NUMA-aware buffer、zero-copy batch）。

---

### O9 (Archive, 2026-06-24) — 异步串口 ISR + RingBuffer + Waker 集成

**原始 Requirement**：

- **WHEN** StarryOS Q6 阶段实现高性能异步串口
- **THEN** 当前影响：uart_16550 API 全部同步（`try_*` 非阻塞 + `*_exact` 自旋 100% CPU），多任务环境不友好，无自然超时
- **AND** 建议方案：在 StarryOS 侧封装 AsyncUart wrapper（路径 B，详见 `.claude/analysis/embassy-integration.md`），三件套：RingBuffer + AtomicWaker + ISR
- **AND** 关联决策：D1（wrapper 位置）+ D2（自研 vs embassy）+ D3（暂不提 PR）
- **AND** 关联 ADR：`architecture/spec.md` Requirement: 异步集成边界
- **AND** 优先级：高 | 触发条件：StarryOS Q6 阶段启动时
- **AND** 预期收益：空闲 CPU 占用从 100% 降至 ~0%，支持自然超时，多任务公平调度

**归档原因**：Q15 M0-M4 全部完成（commit `0393b22` `M3 TtyWrite short-write contract` 为最新），65 tests GREEN，QEMU Manual QA 无退化。

**Q15 完成清单**：
- M0: telemetry feature (commit `eba7f9b`)
- M2: TX completion drain + M4 IER single owner (commit `9e0574e`)
- M3: TtyWrite short-write contract (commit `0393b22`)

**验证**：
- `src/async_/telemetry.rs:25` `pub tx_no_progress: AtomicU64`
- `src/async_/driver.rs:334` `// Budget exhausted — register waker, enable THRE, final recheck`

**恢复条件**：若上游 rust-osdev 拒绝 `async` feature，需 wrapper 回退到路径 A（库内 embassy 依赖）。

---

## 压缩保留区

（无）
