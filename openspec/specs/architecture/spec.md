# architecture/spec.md - 架构决策记录

> Version: 0.6.0
> Last updated: 2026-06-20
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

### Requirement: 异步集成边界（Embassy / Waker / RingBuffer 模式）

库 MUST 保持同步原语，异步适配 MUST 由调用方在 wrapper 层实现，不应将 async runtime 引入库内。

#### Scenario: 集成 embassy 时的职责划分

- **WHEN** 开发者考虑在 StarryOS 等 OS 中实现异步串口
- **THEN** 必须由调用方实现：RingBuffer 缓冲 + AtomicWaker 唤醒 + ISR 数据搬运
- **AND** 库仅提供同步寄存器原语（`try_*` / `isr()` / `ier()`）供 ISR 上下文调用
- **AND** 不应在 uart_16550 库内添加 `embassy` 依赖（违反"零外部 runtime 依赖"原则）

#### Scenario: 选择异步集成路径

- **WHEN** 开发者有三种集成路径选择
- **THEN** 推荐：路径 B（在 StarryOS wrapper 层封装），不修改 uart_16550
- **AND** 次选：路径 C（上游添加 `embedded-io-async` feature），需 PR 说服维护者
- **AND** 不推荐：路径 A（在 uart_16550 内依赖 embassy），与库哲学冲突且社区接受度低

#### Scenario: 验证同步/异步分层正确性

- **WHEN** 评审异步适配代码
- **THEN** 检查：ISR 上下文是否只调用 `try_*` 系列（无 spin_loop）
- **AND** 检查：Future 实现是否使用 ring buffer + Waker 唤醒（无 busy-wait）
- **AND** 检查：FIFO 触发级别是否合理（14 字节为推荐默认）

### Requirement: 异步集成决策记录（2026-06-05）

本次探究产生的 3 个核心决策 MUST 显式记录在 spec 中，作为 StarryOS Q6 阶段的实施依据。

#### Scenario: 决策 D1 - 集成位置选择

- **WHEN** StarryOS 启动 Q6 阶段异步串口工作
- **THEN** **决策**：在 StarryOS wrapper 层封装（路径 B），不修改 uart_16550
- **AND** **原因**：避免上游阻力、保留库零依赖哲学、便于 StarryOS 特定优化（PLIC 中断、DMA）
- **AND** **影响**：StarryOS 侧维护代码、升级 uart_16550 时需保持 wrapper API 稳定
- **AND** **替代方案**：路径 A（库内 embassy feature）已拒绝；路径 C（上游 embedded-io-async）列为次选 PR 候选

#### Scenario: 决策 D2 - async runtime 选择

- **WHEN** 设计 AsyncUart wrapper 时选择 Waker 实现
- **THEN** **决策**：优先基于 `core::task::Waker` 自研 AtomicWaker，**不强制引入 embassy**
- **AND** **原因**：`core::task::Waker` 是 std/core 原语、零额外依赖、避免 embassy 生态耦合
- **AND** **影响**：代码量略增（~50 行 AtomicWaker），但完全可控；后续若需 embassy executor 可平滑切换
- **AND** **替代方案**：直接用 `embassy_sync::waitqueue::AtomicWaker`（如已决定引 embassy 依赖）

#### Scenario: 决策 D3 - 上游协同策略

- **WHEN** 评估是否给 uart_16550 上游提 PR
- **THEN** **决策**：**暂不主动提 PR**，但产出 issue/discussion 文档以备后用
- **AND** **原因**：Q6 阶段 StarryOS 自身需求优先；上游社区对 async 集成持保守态度
- **AND** **影响**：保留 PR 可能性但不阻塞 Q6 进度
- **AND** **触发条件**：如 StarryOS 异步串口落地效果良好，提取 `embedded-io-async` feature 提案给上游

<!-- A1 -->
### Requirement: 异步 UART 状态按端口隔离并编码端点所有权（2026-06-20）

异步 UART 的 IRQ 状态 MUST 按端口隔离，ring endpoint 的并发模型 MUST 由类型所有权编码。

#### Scenario: 设计 IRQ 与 ring 状态

- **WHEN** 异步 UART 支持一个或多个端口及 SMP 执行
- **THEN** **决策**：waker、IER cache、IRQ register accessor 和 drain 状态 MUST 属于具体端口实例
- **AND** **决策**：RX/TX ring MUST 通过唯一 producer/consumer capability 表达 SPSC 所有权
- **AND** **原因**：全局 waker 会产生跨端口串扰，公开 `UnsafeCell` wrapper 会允许并发可变别名，load/store IER 更新会丢位
- **AND** **影响**：ISR 入口需接收 per-port state；reader/writer 构造与 copier 启动 API 需要收紧；多 writer 需显式串行化
- **AND** **替代方案**：继续依赖调用方保证单端口/SPSC 已拒绝，因为约束无法被编译器或测试稳定验证
