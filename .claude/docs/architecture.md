# architecture.md — 架构决策记录

> 由 project-rules-generator 初始化，由 project-docs-assistant 日常维护。
> 条目格式: <!-- A{编号} --> ### {DATE} - {决策标题}，每条含决策、原因、影响、替代方案。

---

<!-- A1 --> ### 2026-05 - Backend trait 泛型架构

**决策**: 使用 `Backend` trait + 泛型参数 `Uart16550<B: Backend>` 而非运行时枚举分派

**原因**:
- PIO 和 MMIO 的地址类型完全不同 (u16 vs NonNull<u8>)，无法统一为单一枚举
- 泛型在编译时单态化，零运行时开销，对嵌入式/OS 场景至关重要
- Sealed trait 防止外部实现，保证 API 稳定性

**影响**:
- 用户代码需在编译时选择后端类型
- 无法在运行时切换 PIO/MMIO
- 代码体积略增（单态化两份代码），但可接受

**替代方案**:
- 运行时枚举分派: 需要统一地址类型，引入额外分支开销
- 宏生成: 过于复杂，可读性差

---

<!-- A2 --> ### 2026-05 - Sealed Trait 模式

**决策**: `Backend` 和 `RegisterAddress` 使用 sealed trait（private::Sealed）

**原因**:
- 防止 crate 外部实现这些 trait，避免破坏性变更
- 仅 crate 内部提供 MmioBackend 和 PioBackend 实现

**影响**:
- 用户无法自定义 Backend 实现
- 如需新后端，必须在本 crate 内添加

**替代方案**:
- 开放 trait: 允许外部实现，但需保证 API 稳定性承诺
- 使用 enum 替代 trait: 丧失泛型优势

---

<!-- A3 --> ### 2026-05 - NonNull MMIO 地址要求

**决策**: `new_mmio()` 构造函数要求 `NonNull<u8>` 而非 `*mut u8`

**原因**:
- 空指针在 MMIO 场景下无意义，是编程错误
- NonNull 提供类型级别保证，消除运行时 null 检查
- 与 Rust 指针安全模型一致

**影响**:
- 调用者必须先检查指针非空
- API 更安全，不可能构造空 MMIO 设备

**替代方案**:
- 接受 `*mut u8` 并返回 Err: 运行时检查，但类型系统无法阻止

---

<!-- A4 --> ### 2026-05 - aarch64 内联汇编 MMIO

**决策**: 在 aarch64 上使用 `ldrb`/`strb` 内联汇编替代 `ptr::read_volatile`

**原因**:
- LLVM 在 aarch64 可能发出 LDR 指令，在虚拟化环境下不可正确虚拟化
- 详见 rust-lang/rust#131894

**影响**:
- aarch64 路径使用 `core::arch::asm!`，需要 nightly 或已稳定的 asm 特性
- 其他架构仍使用标准 volatile read/write

**替代方案**:
- 使用 `core::ptr::read_volatile`: 在某些虚拟化环境下可能失败
- 使用 `core::arch::asm!` 对所有架构: 不必要，x86 volatile 已足够

---

<!-- A5 --> ### 2026-05 - bitflags + 便捷枚举双层设计

**决策**: 寄存器使用 bitflags 宏定义原始位操作，同时提供 WordLength/Parity 等便捷枚举

**原因**:
- bitflags 提供 ABI 兼容的原始位操作（与硬件寄存器一一对应）
- 便捷枚举提供类型安全和可读性
- 通过 from_raw_bits/to_raw_bits 桥接两层

**影响**:
- 代码量增加（每寄存器需 bitflags + 枚举 + 转换函数）
- 测试需覆盖两层 round-trip

**替代方案**:
- 仅 bitflags: 可读性差，用户需手动构造位模式
- 仅枚举: 丧失原始位操作能力，无法处理保留位