# architecture.md — 架构决策记录

> 由 project-rules-generator 初始化，由 project-docs-assistant 日常维护。
> 条目格式: <!-- A{编号} --> ### {DATE} - {决策标题}，每条含决策、原因、影响、替代方案。

---

<!-- A1 --> ### 2026-05-25 - 后端抽象架构 (Backend trait)

**决策**: 使用 sealed Backend trait 抽象 I/O 访问方式，两种实现: PioBackend (x86) 和 MmioBackend (通用)

**原因**:
- 16550 芯片可挂载在 x86 PIO 总线或 MMIO 总线上
- 同一芯片不同访问方式，仅底层读写不同
- sealed trait 防止外部实现，保证类型安全和行为一致

**影响**:
- Uart16550<B: Backend> 的泛型参数在编译期决定后端
- PioBackend 仅 cfg(x86/x86_64) 编译，RISC-V 只能用 MmioBackend
- MMIO 需要 NonNull<u8> + stride，PIO 只需 u16 端口号

**替代方案**:
- 运行时分发 (dyn trait) → 拒绝: 增加运行时开销，no_std 场景下不必要
- 宏生成两个版本 → 拒绝: 代码重复，维护成本高

---

<!-- A2 --> ### 2026-05-25 - 寄存器访问分层设计

**决策**: 寄存器定义分为两层: bitflags (原始位操作) + 便捷枚举 (类型安全)

**结构**:
- `spec.rs` 定义 bitflags: IER, ISR, FCR, LCR, LSR, MCR, MSR
- `spec.rs` 定义枚举: WordLength, Parity, FifoTriggerLevel, BaudRate
- 枚举通过 from_raw_bits/to_raw_bits 与 bitflags 双向转换
- Config struct 使用枚举，底层写入寄存器时转为 bitflags

**原因**:
- bitflags 提供完整的 ABI 兼容原始位操作（中断处理、调试需要）
- 枚举提供类型安全的配置接口（编译期检查）
- 双层设计满足两种使用场景

---

<!-- A3 --> ### 2026-05-25 - TTY 封装与 fmt::Write

**决策**: Uart16550Tty 封装 Uart16550 并实现 core::fmt::Write

**原因**:
- 内核早期启动需要 print!/debug! 输出，依赖 fmt::Write
- 自动 \n→\r\n 转换，符合终端协议
- 与 Uart16550 分离，避免主类型承担格式化职责

**影响**:
- Uart16550Tty::new_port/new_mmio 自动调用 init() + loopback 测试
- 失败直接 panic，适用于早期启动不可恢复场景

**替代方案**:
- 直接在 Uart16550 上实现 fmt::Write → 拒绝: 违反单一职责，且 init 结果需处理

---

<!-- A4 --> ### 2026-05-25 - aarch64 MMIO 内联汇编

**决策**: aarch64 后端使用 `ldrb`/`strb` 内联汇编替代 `ptr::read_volatile`

**原因**:
- LLVM 可能发出不可虚拟化的 MMIO 读取指令
- 在 QEMU/KVM 等虚拟化环境下可能导致异常
- 内联汇编保证生成预期的单字节 load/store 指令

**影响**:
- aarch64 和其他平台使用不同实现（#[cfg(target_arch)] 分发）
- 其他平台 (RISC-V 等) 仍使用 ptr::read_volatile

---

<!-- A5 --> ### 2026-05-25 - embedded-io 适配层

**决策**: 可选 feature "embedded-io"，为 Uart16550 实现 Read/Write/ReadReady/WriteReady

**原因**:
- embedded-io 是 embedded 生态的标准 I/O trait
- 支持 embedded-io async 特性，为异步运行时集成预留
- 可选 feature 避免强制依赖

**影响**:
- 启用 feature 后 Uart16550 可直接用于 embedded-io 消费者
- 实现为自旋阻塞模式 (receive_bytes_exact/send_bytes_exact)
- 异步集成需要外部运行时驱动

---

<!-- A6 --> ### 2026-05-25 - 非 DMA 模式限制

**现状**: 库支持 IER::DMA_RX_END / DMA_TX_END 位和 InterruptType 枚举值，但无 DMA 传输 API

**原因**: DMA 传输需要硬件特定的 DMA 控制器集成，超出驱动库职责

**影响**: StarryOS 若需 DMA 串口，需自行集成 DMA 控制器，利用 IER 中断位配合
