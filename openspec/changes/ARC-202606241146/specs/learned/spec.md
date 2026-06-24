# learned 域归档条目

> Carrier spec for ARC-202606241146
> 源文档: openspec/specs/learned/spec.md

---

## 完整保留区

### L_EXP-3 (Archive, 2026-06-24) — 待探索清单第 3 项

**原始 Requirement 内容**（位于源文档 `### Requirement: 待探索清单` Scenario 中）：

- 硬件测试子项目（test/）的启动流程和调试接口用法
- `Config::prescaler_division_factor` 的实际硬件使用场景
- **embedded-io feature 在 StarryOS 中的集成方式** ← 本条归档

**归档原因**：StarryOS 已通过 `async` feature（而非 `embedded-io` feature）完成集成。

**关键 commit**：
- `f260fde` `feat(uart-async): migrate device_ops (AsyncUartReader/Writer) to uart_16550`
- `688f605` `feat(uart-async): migrate copier driver to uart_16550 with NAPI coalescing`
- `4c8c1ac` `feat(uart-async): migrate ring buffer to uart_16550 with generic OsWakerSet`
- `c231ab7` `feat(uart-async): migrate ISR handler to uart_16550 with AtomicWaker pattern`

**剩余未归档**：前 2 项（硬件测试子项目 + prescaler_division_factor）保留在源文档。

**恢复条件**：若 StarryOS 需要从 `async` feature 降级到 `embedded-io` feature（unlikely）。

---

## 压缩保留区

（无）
