# tasks 域归档条目

> Carrier spec for ARC-202606241146
> 源文档: .claude/docs/tasks.md

---

## 完整保留区

### T31 (Archive, 2026-06-24) — 性能退化阻塞项

**原始内容**（位于源文档 `## 阻塞项` 区域）：

```
- **⚠️ 性能退化**: write+tcdrain benchmark 5.4x 开销。RingBufTx::push() 每调用获取 SpinNoIrq + RefCell::borrow_mut，ISR handle_irq 路径同样获取 SpinNoIrq（ArceOsUartPort 方法）。两路径竞争同一 SpinNoIrq 实例，形成锁竞争退化。待优化方向：(1) Mutex 内 RefCell→UnsafeCell (2) handle_irq 单次锁复用。
```

**归档原因**：用户陈述已完成；Q15 M0-M4 已通过 batch push/pop + inline-always + 单一 SpinNoIrq 实例缓解原 5.4x 开销。

**关键 commit**：
- `73aca5c` `perf(uart-async): add batch push/pop to reduce lock overhead`
- `a0cead0` `perf(uart-async): add #[inline(always)] to ring buffer push/pop`
- `9e0574e` `feat(uart-async): Q15-M2 TX completion drain + M4 IER single owner`

**当前锁结构验证**：
- `src/async_/driver.rs:41` 单点 SpinNoIrq
- `src/async_/ring_buffer.rs` 使用 `embassy_hal_internal` 无锁 SPSC
- batch push/pop 减少锁获取次数

**恢复条件**：若 benchmark 重新出现 5.4x 开销，需检查 `RingBufTx::push_batch` 路径是否引入新锁点。

---

## 压缩保留区

（无）
