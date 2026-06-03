# architecture/spec.md - 架构决策记录

> Version: 0.6.0
> Last updated: 2026-06-03
> Migrated from: .claude/docs/architecture.md (2026-05-25)

## Purpose

定义 uart_16550 项目的架构决策和设计原则，指导开发过程中的技术选型和系统设计。所有重要决策以 ADR 形式记录，包含决策内容、原因、影响和替代方案。

## Requirements

### Requirement: 后端抽象架构（Backend trait）

所有 Uart16550 实例 MUST 使用 sealed Backend trait 抽象 I/O 访问方式，提供 PioBackend (x86) 和 MmioBackend (通用) 两种实现。

#### Scenario: 选择后端实现

- **WHEN** 开发者创建 Uart16550 实例
- **THEN** 必须在编译期通过泛型参数 `<B: Backend>` 选择后端（`new_port()` → PioBackend，`new_mmio()` → MmioBackend）
- **AND** PioBackend 仅在 `cfg(target_arch = "x86" or "x86_64")` 下编译，RISC-V 等架构只能使用 MmioBackend

#### Scenario: 评估后端抽象设计

- **WHEN** 开发者评估是否引入运行时分发 (dyn trait) 或宏生成
- **THEN** 应当拒绝：增加运行时开销（no_std 场景下不必要）或代码重复（维护成本高）
- **AND** 编译期泛型分发是首选方案

### Requirement: 寄存器访问分层设计

所有 16550 寄存器定义 MUST 分为两层：bitflags（原始位操作）+ 便捷枚举（类型安全）。

#### Scenario: 选择寄存器抽象层级

- **WHEN** 开发者需要操作 16550 寄存器
- **THEN** 在 `spec.rs` 中：bitflags 类型（IER/ISR/FCR/LCR/LSR/MCR/MSR）提供 ABI 兼容的原始位操作
- **AND** 在 `spec.rs` 中：枚举类型（WordLength/Parity/FifoTriggerLevel/BaudRate）提供类型安全的配置接口
- **AND** 枚举通过 `from_raw_bits()`/`to_raw_bits()` 与 bitflags 双向转换
- **AND** `Config` struct 使用枚举，底层写入寄存器时转为 bitflags

#### Scenario: 决定使用 bitflags 还是枚举

- **WHEN** 开发者需要中断处理或调试场景
- **THEN** 使用 bitflags 获得完整 ABI 兼容的原始位操作
- **WHEN** 开发者需要类型安全的配置接口
- **THEN** 使用便捷枚举获得编译期检查

### Requirement: TTY 封装与 fmt::Write

Uart16550Tty MUST 封装 Uart16550 并实现 core::fmt::Write，支持内核早期启动的 print!/debug! 输出。

#### Scenario: 使用 TTY 封装

- **WHEN** 开发者需要在内核早期启动阶段输出日志
- **THEN** 可以使用 `Uart16550Tty::new_port()` 或 `new_mmio()` 构造
- **AND** 构造时自动调用 `init()` 并执行 loopback 测试
- **AND** 失败时直接 panic（适用于早期启动不可恢复场景）
- **AND** 写入自动执行 `\n` → `\r\n` 转换，符合终端协议

#### Scenario: 评估 TTY 分离设计

- **WHEN** 开发者评估是否在 Uart16550 上直接实现 fmt::Write
- **THEN** 应当拒绝：违反单一职责，且 init 结果需处理
- **AND** 通过 Uart16550Tty 分离格式化职责

### Requirement: aarch64 MMIO 内联汇编

aarch64 后端 MUST 使用 `ldrb`/`strb` 内联汇编替代 `ptr::read_volatile`，以避免 LLVM 发出不可虚拟化的 MMIO 指令。

#### Scenario: 在 aarch64 平台实现 MMIO 访问

- **WHEN** 开发者为 aarch64 平台编写 MMIO 后端
- **THEN** 必须使用 `ldrb`/`strb` 内联汇编（在 `src/backend/mmio.rs:arch` 模块）
- **AND** 不能使用 `ptr::read_volatile`，因 LLVM 可能发出不可虚拟化的 MMIO 读取指令
- **AND** 在 QEMU/KVM 等虚拟化环境下可避免异常

#### Scenario: 选择其他架构的 MMIO 实现

- **WHEN** 开发者为 RISC-V/x86 等其他平台编写 MMIO 后端
- **THEN** 可使用 `ptr::read_volatile`（非 aarch64 平台不受此约束）

### Requirement: embedded-io 适配层

可选 feature "embedded-io" MUST 为 Uart16550 实现 Read/Write/ReadReady/WriteReady trait，遵循 embedded-io 生态标准。

#### Scenario: 启用 embedded-io feature

- **WHEN** 开发者启用 `embedded-io` feature
- **THEN** Uart16550 可直接用于 embedded-io 消费者
- **AND** 实现为自旋阻塞模式（`receive_bytes_exact()`/`send_bytes_exact()`）
- **AND** 异步集成需要外部运行时驱动（库本身不提供 async API）

#### Scenario: 选择是否启用 feature

- **WHEN** 开发者不需要 embedded-io 集成
- **THEN** 可以不启用 feature，避免强制依赖

### Requirement: 非 DMA 模式限制

库 MUST 支持 IER::DMA_RX_END / DMA_TX_END 位和 InterruptType 枚举值，但 MUST NOT 提供 DMA 传输 API（由调用方集成 DMA 控制器）。

#### Scenario: 评估 DMA 支持范围

- **WHEN** 开发者需要 DMA 串口传输
- **THEN** 库提供 DMA 中断位（IER）和寄存器位（FCR::DMA_MODE/ENABLE_DMA_END）支持
- **AND** 但不提供 DMA 传输控制 API（需要硬件特定的 DMA 控制器集成）
- **AND** StarryOS 若需 DMA 串口，需自行集成 DMA 控制器，利用 IER 中断位配合

#### Scenario: 评估是否扩展 DMA API

- **WHEN** 开发者考虑添加 DMA 传输 API
- **THEN** 应参考 optimization.md 中的 O8 优化项（DMA 模式寄存器完整控制）
