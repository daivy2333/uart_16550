# learned/spec.md - 项目学习记忆

> Version: 0.6.0
> Last updated: 2026-06-20 (UART correctness audit: ownership/waker/ISR invariants)
> Migrated from: .claude/docs/learned.md (2026-05-25)

## Purpose

记录项目开发过程中学到的知识，避免重复探索，加速问题解决。涵盖 API 路径、文件速查、寄存器速查、踩坑档案、技巧模式、依赖关系图。

## Requirements

### Requirement: API 路径记录

项目中使用的关键 API 路径 MUST 记录在 spec 中，包含用途、签名和使用示例。

#### Scenario: 查找构造 API

- **WHEN** 开发者需要构造 Uart16550 实例
- **THEN** 可以在 `src/lib.rs` 找到：
  - `Uart16550::new_port(base_port: u16)` - unsafe fn, 创建 x86 PIO 实例
  - `Uart16550::new_mmio(addr: NonNull<u8>, stride: u8)` - unsafe fn, 创建 MMIO 实例
  - `Uart16550::init(&mut self, config: Config) -> Result<(), InitError>`
- **AND** 在 `src/tty.rs` 找到便捷构造：
  - `Uart16550Tty::new_port()` - TTY PIO 便捷构造, 自动 init + loopback test
  - `Uart16550Tty::new_mmio()` - TTY MMIO 便捷构造, 自动 init + loopback test

#### Scenario: 查找数据 I/O API

- **WHEN** 开发者需要读写数据
- **THEN** 可以使用：
  - `try_receive_byte(&mut self) -> Result<u8, ByteReceiveError>` - 非阻塞读一字节
  - `try_send_byte(&mut self, byte: u8) -> Result<(), ByteSendError>` - 非阻塞写一字节
  - `receive_bytes(&mut self, &mut [u8]) -> usize` - 非阻塞读尽可能多字节
  - `send_bytes(&mut self, &[u8]) -> usize` - 非阻塞写尽可能多字节
  - `receive_bytes_exact(&mut self, &mut [u8])` - 自旋阻塞直到读满
  - `send_bytes_exact(&mut self, &[u8])` - 自旋阻塞直到写完

#### Scenario: 查找状态查询 API

- **WHEN** 开发者需要查询设备状态
- **THEN** 可以使用：
  - `ready_to_receive()` - 检查 LSR::DATA_READY
  - `ready_to_send()` - 检查 LSR::THR_EMPTY
  - `test_loopback()` - 环回自检
  - `check_connected()` - 检查远程连接

#### Scenario: 查找寄存器读取 API

- **WHEN** 开发者需要读取寄存器
- **THEN** 可以使用：
  - `ier()` / `isr()` / `lcr()` / `mcr()` / `lsr()` / `msr()` / `spr()` - 读单个寄存器
  - `dll_dlm()` - 读除数锁存器（临时设 DLAB）
  - `config_register_dump()` - 一次读取所有配置寄存器

#### Scenario: 查找配置 API

- **WHEN** 开发者需要查看默认配置
- **THEN** 可以使用 `Config::DEFAULT`（8-N-1, 9600 波特率, FIFO 触发 14, IER::DATA_READY）

### Requirement: 文件速查表

关键文件和目录的位置 MUST 记录在 spec 中，加速代码导航。

#### Scenario: 定位关键文件

- **WHEN** 开发者需要修改 16550 寄存器定义
- **THEN** 应编辑 `src/spec.rs`（bitflags + 常量 + 计算函数 + InterruptType）
- **WHEN** 开发者需要添加错误类型
- **THEN** 应编辑 `src/error.rs`（InvalidAddressError/InitError/LoopbackError/ByteReceiveError/ByteSendError/RemoteReadyToReceiveError）
- **WHEN** 开发者需要修改 MMIO 后端
- **THEN** 应编辑 `src/backend/mmio.rs`（MmioBackend, MmioAddress, aarch64 内联汇编）
- **WHEN** 开发者需要修改 PIO 后端
- **THEN** 应编辑 `src/backend/pio.rs`（PioBackend, PortIoAddress, x86 inb/outb）
- **WHEN** 开发者需要修改 embedded-io 适配
- **THEN** 应编辑 `src/embedded_io.rs`
- **WHEN** 开发者需要修改配置
- **THEN** 应编辑 `src/config.rs`（Config struct + BaudRate enum + 默认值）
- **WHEN** 开发者需要修改 TTY 封装
- **THEN** 应编辑 `src/tty.rs`（Uart16550Tty, impl fmt::Write, 自动 \n→\r\n）

### Requirement: 寄存器速查表

16550 寄存器布局（偏移、读/写/特殊功能、说明） MUST 可通过 spec 速查。

#### Scenario: 查询寄存器功能

- **WHEN** 开发者需要了解寄存器布局
- **THEN** 应参考以下表：
  - 偏移 0: RHR/THR/DLL (数据/除数低位)
  - 偏移 1: IER/DLM (中断启用/除数高位)
  - 偏移 2: ISR/FCR (中断状态/FIFO 控制)
  - 偏移 3: LCR (线路控制，含 DLAB 位)
  - 偏移 4: MCR (Modem 控制)
  - 偏移 5: LSR (线路状态，只读)
  - 偏移 6: MSR/PSD (Modem 状态/预分频器)
  - 偏移 7: SPR (便笺簿)

### Requirement: 中断速查表

ISR IIC 解码和 IER 启用位 MUST 可通过 spec 速查。

#### Scenario: 解码中断类型

- **WHEN** 开发者收到中断并读取 ISR
- **THEN** 可以按 IIC (bits[3:1]) 解码为 7 种 InterruptType：
  - 0b011: ReceiverLineStatus (优先级 1, 清除方式：读 LSR)
  - 0b010: ReceivedDataReady (优先级 2, 清除方式：读 RHR)
  - 0b110: ReceptionTimeout (优先级 2, 清除方式：读 RHR)
  - 0b001: TransmitterHoldingRegisterEmpty (优先级 3, 清除方式：写数据或读 ISR)
  - 0b000: ModemStatus (优先级 4, 清除方式：读 MSR)
  - 0b111: DmaReceptionEndOfTransfer (优先级 5, 清除方式：读 ISR，非标准)
  - 0b101: DmaTransmissionEndOfTransfer (优先级 6, 清除方式：读 ISR，非标准)

#### Scenario: 启用特定中断

- **WHEN** 开发者需要启用特定中断
- **THEN** 可在 IER 中设置以下位：
  - 0: DATA_READY (数据就绪)
  - 1: THR_EMPTY (THR 空)
  - 2: RECEIVER_LINE_STATUS (线路状态错误)
  - 3: MODEM_STATUS (Modem 状态变更)
  - 6: DMA_RX_END (DMA 接收结束，非标准)
  - 7: DMA_TX_END (DMA 传输结束，非标准)

### Requirement: FIFO 速查表

FIFO 触发级别和关键常量 MUST 可通过 spec 速查。

#### Scenario: 配置 FIFO 触发级别

- **WHEN** 开发者需要配置 FIFO 触发级别
- **THEN** 可以选择 4 个 FifoTriggerLevel 值：
  - One (0b00): 1 字符，每字符中断
  - Four (0b01): 4 字符
  - Eight (0b10): 8 字符
  - Fourteen (0b11): 14 字符，默认/推荐

#### Scenario: 引用 FIFO 常量

- **WHEN** 开发者需要 FIFO 大小和时钟频率
- **THEN** 可以使用 `FIFO_SIZE = 16` 和 `CLK_FREQUENCY_HZ = 1_843_200`

### Requirement: Config 字段说明

Config struct 的字段、类型、默认值 MUST 可通过 spec 速查。

#### Scenario: 配置 Uart16550

- **WHEN** 开发者构造 Config
- **THEN** 各字段及其默认值如下：
  - `interrupts: IER` (默认 DATA_READY) - 中断启用掩码
  - `frequency: u32` (默认 1_843_200) - 时钟频率 Hz
  - `prescaler_division_factor: Option<u32>` (默认 None) - 预分频器
  - `fifo_trigger_level: Option<FifoTriggerLevel>` (默认 Some(Fourteen)) - None 表示禁用 FIFO
  - `baud_rate: BaudRate` (默认 Baud9600) - 波特率
  - `data_bits: WordLength` (默认 EightBits) - 数据位
  - `extra_stop_bits: bool` (默认 false) - 额外停止位
  - `parity: Parity` (默认 Disabled) - 校验

### Requirement: 波特率计算公式

波特率、除数、频率的相互转换公式 MUST 记录在 spec 中。

#### Scenario: 计算波特率参数

- **WHEN** 开发者需要计算串口参数
- **THEN** 公式为 `baud_rate = frequency / (16 * (prescaler + 1) * divisor)`
- **AND** 可使用以下 API：
  - `calc_baud_rate(frequency, divisor, prescaler) -> Result<u32, NonIntegerBaudRateError>`
  - `calc_frequency(baud_rate, divisor, prescaler) -> u32`
  - `calc_divisor(frequency, baud_rate, prescaler) -> Result<u16, NonIntegerDivisorError>`

### Requirement: init() 执行序列

`Uart16550::init()` 的 9 步执行流程 MUST 记录在 spec 中。

#### Scenario: 理解 init 流程

- **WHEN** 开发者需要理解 init 流程
- **THEN** 应了解 9 步序列：
  1. 保存 config
  2. SPR 存在测试（写 0x42/0x73 回读）
  3. IER=0 禁用中断
  4. 设波特率（LCR=DLAB, 写 DLL+DLM, LCR=0）
  5. 设 LCR（字长+停止位+校验）
  6. 配置 FCR（FIFO 启用/重置/触发级别）
  7. 设 MCR（DTR+RTS+OUT_2_INT_ENABLE）
  8. 按 config.interrupts 启用中断
  9. 等待 THR/FIFO/TSR 清空

### Requirement: 踩坑档案

遇到的技术陷阱和解决方案 MUST 记录在 spec 中，防止重复踩坑。

#### Scenario: aarch64 MMIO 不可虚拟化指令

- **WHEN** 开发者在 aarch64 平台遇到 MMIO 读取异常
- **THEN** 应意识到：LLVM 可能发出不可虚拟化的 MMIO 读取指令
- **AND** 解法：使用 `ldrb`/`strb` 内联汇编替代 `ptr::read_volatile`（见 `src/backend/mmio.rs:arch` 模块）

#### Scenario: QEMU FIFO 禁用死循环

- **WHEN** 开发者在 QEMU 中设置 `fifo_trigger_level: None` 遇到数据无法 drain
- **THEN** 应意识到：QEMU 设备模型在非 FIFO 模式下不正确处理 THR drain
- **AND** 解法：建议始终启用 FIFO（`Config::DEFAULT` 使用 `FifoTriggerLevel::Fourteen`）

#### Scenario: MCR::OUT_2_INT_ENABLE 中断路由

- **WHEN** 开发者发现 UART 中断不触发
- **THEN** 应检查 MCR::OUT_2_INT_ENABLE 是否设置（典型 x86 系统中此位将 UART IRQ 线连接到 PIC/APIC）
- **AND** `init()` 已自动设置，但自定义初始化流程需注意

#### Scenario: RISC-V 平台后端选择

- **WHEN** 开发者在 RISC-V 平台使用 PioBackend
- **THEN** 应当切换到 `Uart16550::new_mmio(base_addr, stride)`，因 PioBackend 仅在 `cfg(x86/x86_64)` 下编译

### Requirement: 技巧模式

有效的开发技巧和模式 MUST 记录在 spec 中。

#### Scenario: Sealed Trait 模式

- **WHEN** 开发者需要限制外部实现 Backend 或 RegisterAddress
- **THEN** 应使用 sealed trait 模式：`mod private { pub trait Sealed {} }`
- **AND** 保证 API 稳定性，防止不兼容的外部实现

#### Scenario: bitflags + 便捷枚举双层设计

- **WHEN** 开发者需要寄存器操作
- **THEN** 应使用 bitflags 提供 ABI 兼容的原始位操作
- **AND** 同时提供 WordLength/Parity/FifoTriggerLevel 等便捷枚举，通过 `from_raw_bits`/`to_raw_bits` 转换

#### Scenario: 中断处理模式

- **WHEN** 开发者编写中断处理程序
- **THEN** 应遵循以下模式：读 ISR → `isr.interrupt_type()` 匹配 InterruptType → 按类型执行对应操作清除中断
- **AND** ReceivedDataReady: 读 RHR; THR_EMPTY: 写数据; ReceiverLineStatus: 读 LSR; ModemStatus: 读 MSR

#### Scenario: MMIO stride 参数选择

- **WHEN** 开发者需要为 SoC 配置 MMIO stride
- **THEN** 典型值：1（x86 PC）, 4（ARM/SoC 按字对齐）, RISC-V QEMU virt 机器通常 1

### Requirement: 依赖关系图

核心模块依赖关系和类型关系 MUST 记录在 spec 中。

⚠️ STALE [2026-06-24] — 自 2026-05-25 迁移以来未更新，async feature 5 个新模块路径（async_/isr.rs / ring_buffer.rs / driver.rs / device_ops.rs / os/mod.rs）未含。建议 30 天内补齐或 Archive。

#### Scenario: 理解核心依赖链

- **WHEN** 开发者需要理解模块依赖
- **THEN** 应了解：
  - `lib.rs` → `config.rs` → `spec.rs` (BaudRate, 常量)
  - `lib.rs` → `error.rs` → `spec.rs` (NonIntegerDivisorError)
  - `lib.rs` → `backend/mod.rs` → `backend/mmio.rs`, `backend/pio.rs`
  - `tty.rs` → `lib.rs` (Uart16550)
  - `embedded_io.rs` → `lib.rs` (Uart16550) + embedded-io crate

#### Scenario: 理解类型关系

- **WHEN** 开发者需要理解类型结构
- **THEN** 应了解：
  - `Uart16550<B: Backend> { backend: B, base_address: B::Address, config: Config }`
  - `PioBackend: Address=PortIoAddress(u16)`, stride=1（固定）
  - `MmioBackend: Address=MmioAddress(NonNull<u8>)`, stride=用户指定
  - `Uart16550Tty<B>(Uart16550<B>)`: impl fmt::Write, 自动 \n→\r\n

### Requirement: 错误类型速查表

错误类型及其触发场景 MUST 可通过 spec 速查。

#### Scenario: 错误诊断

- **WHEN** 开发者遇到错误
- **THEN** 可以根据错误类型定位：
  - `InvalidAddressError`: InvalidBaseAddress / InvalidStride（构造时地址无效）
  - `InitError`: DeviceNotPresent / InvalidBaudRate（init 失败）
  - `LoopbackError`: SendError / UnexpectedLoopbackByte / UnexpectedLoopbackMsg（环回测试失败）
  - `ByteReceiveError`: 无变体（无数据可读）
  - `ByteSendError`: NoCapacity / RemoteNotClearToSend（FIFO 满 / 远端未就绪）
  - `RemoteReadyToReceiveError`: NoRemoteConnectedNoDSR / NoRemoteConnectedNoCD / RemoteNotClearToSend（远端未连接）

### Requirement: 待探索清单

尚未完全理解或验证的技术点 MUST 记录在 spec 中。

#### Scenario: 探索待办

- **WHEN** 开发者有时间进一步理解项目
- **THEN** 可以关注：
  - 硬件测试子项目（test/）的启动流程和调试接口用法
  - `Config::prescaler_division_factor` 的实际硬件使用场景

### Requirement: 异步适配层 API 速查

异步适配层需要的同步原语 MUST 在 spec 中列出。

#### Scenario: 定位异步适配入口 API

- **WHEN** 开发者实现 AsyncUart wrapper（StarryOS 等）
- **THEN** 应使用以下同步原语（**只允许在 ISR/极短上下文中调用**）：
  - `try_receive_byte()` — `src/lib.rs:699` — ISR 中读单字节，非阻塞
  - `try_send_byte()` — `src/lib.rs:712` — ISR 中写单字节，非阻塞
  - `receive_bytes(buf)` — `src/lib.rs:728` — 批量读直到空（部分阻塞）
  - `send_bytes(buf)` — `src/lib.rs:746` — 批量写直到满（部分阻塞）
  - `isr()` — `src/lib.rs` — 读 ISR，**必须在 ISR 入口先读以确认和清除中断**
  - `ier()` — 读 IER 寄存器（运行时切换中断需要）
  - `InterruptType` — `src/spec.rs` — ISR 分发枚举（7 种）
  - `IER::DATA_READY` / `THR_EMPTY` / `RECEIVER_LINE_STATUS` — 中断位定义

#### Scenario: 避免在 ISR 中使用阻塞 API（踩坑档案）

- **WHEN** 开发者写中断处理函数
- **THEN** **禁止**使用 `receive_bytes_exact` / `send_bytes_exact`（会自旋死锁其他中断）
- **AND** **禁止**使用 `embedded_io::Read::read` / `Write::write`（内部 `hint::spin_loop()`）
- **AND** 只使用 `try_*` 系列（瞬时返回，无等待）
- **AND** 如果 ISR 需要等待更多数据，应 yield 后让出（`return` ISR，等待下次中断）

### Requirement: 异步适配架构模式（标准三件套）

异步适配层的标准模式 MUST 记录为技巧。

#### Scenario: 标准 ISR + RingBuffer + Waker 模式

- **WHEN** 开发者实现异步 UART
- **THEN** 应遵循三组件模式：
  1. **RingBuffer** — 静态数组 + 头尾指针（或 heap），无锁或临界区保护
  2. **AtomicWaker** — `core::task::Waker` 包装，支持并发注册（embassy 提供现成的，自研也简单）
  3. **ISR 入口** — 读 ISR → 分发 InterruptType → 同步搬数据 → `waker.wake()`
- **AND** 不需要引入 embassy 依赖（`core::task::Waker` 是 std/core 原语）
- **AND** 具体代码模式见 `.claude/analysis/embassy-integration.md` 第 3 节
- **AND** StarryOS Q6 阶段已规划此模式（见父 CLAUDE.md）

#### Scenario: FIFO 触发级别选型技巧

- **WHEN** 配置异步串口的 FIFO 触发级别
- **THEN** 默认推荐 `FifoTriggerLevel::Fourteen`（最省中断次数）
- **AND** 低延迟场景可改 `One`（每字节中断）
- **AND** 平衡选择 `Four` 或 `Eight`
- **AND** 在 `Config { interrupts: IER::DATA_READY, ... }` 中控制启用位

### Requirement: Ring/Copier 可观测性指标 API（M4 新增）

Ring buffer 和 copier 的运行时指标 MUST 通过公开 Atomic 字段暴露。

#### Scenario: 查询 ring buffer 指标

- **WHEN** OS 层需要监控 ring buffer 状态
- **THEN** 直接访问 `RingBufRx`/`RingBufTx` 的公开字段（均为 `AtomicUsize`）：
  - `rx.accepted.load(Ordering::Relaxed)` — 已写入字节总数
  - `rx.dropped.load(Ordering::Relaxed)` — 丢弃字节数
  - `rx.high_water.load(Ordering::Relaxed)` — 占用峰值
  - `rx.popped.load(Ordering::Relaxed)` — 已读出字节数
- **AND** 同样适用于 TX ring

#### Scenario: 查询 copier 指标

- **WHEN** OS 层需要监控 copier 效率
- **THEN** 访问 `AsyncUartDriver` 的公开字段（均为 `AtomicU64`）：
  - `driver.rx_poll.load(Relaxed)` / `tx_poll` — poll 次数
  - `driver.rx_hw_bytes.load(Relaxed)` / `tx_hw_bytes` — 硬件字节数
  - `driver.rx_no_progress.load(Relaxed)` / `tx_no_progress` — 空 poll 次数

### Requirement: Waker 注册必须在中断使能之前（M4 修复）

RX/TX copier 的 waker 注册顺序 MUST 遵循先 register 后 enable 原则。

#### Scenario: 正确的 RX copier 注册顺序

- **WHEN** RX copier 进入 NAPI polling 模式
- **THEN** `RX_WAKER.register(cx.waker())` MUST 在 `enable_rx_intr()` 之前调用
- **AND** 反之则存在竞态窗口：中断在 register 之前到达 → waker 丢失 → 数据永久卡住

### Requirement: TX copier 无法取得进展时必须 yield（M4 修复）

TX copier MUST 在 `send_bytes() == 0` 时返回 `Poll::Pending` 而非 `Poll::Ready(())`。

#### Scenario: THR 满时正确挂起

- **WHEN** `send_bytes()` 返回 0（发送保持寄存器满）
- **THEN** copier MUST 返回 `Poll::Pending` 等待 TX 中断唤醒
- **AND** 返回 `Poll::Ready(())` 会导致 busy-poll 紧循环浪费 CPU

<!-- L1 -->
### Requirement: 异步条件等待必须 register 后重新检查

异步条件等待 MUST 在注册 waker 后重新检查条件；单纯“先 register、后 enable”不足以覆盖所有丢唤醒竞态。

#### Scenario: ring 判空后进入 Pending

- **WHEN** consumer 因 ring 为空准备返回 `Poll::Pending`
- **THEN** MUST 先注册 waker，再重新检查 ring 是否仍为空
- **AND** 禁止使用“先判空、后 register、直接 Pending”的顺序
- **AND** UART 中断等待同样应使用 register → enable → condition recheck 的闭环

<!-- L2 -->
### Requirement: SPSC 安全前提必须由类型所有权保证

SPSC producer/consumer 的唯一性 MUST 由类型所有权保证；`UnsafeCell<Reader/Writer>` 的安全性不能依赖公开 API 调用者自律。

#### Scenario: 暴露 ring endpoint

- **WHEN** ring wrapper 通过 `&self` 取得内部 `&mut Reader/Writer`
- **THEN** producer 和 consumer capability MUST 各自唯一且不可复制
- **AND** 禁止为可 Clone/可重复构造的 wrapper 无条件实现 `Sync`
- **AND** 多生产者或多消费者需求必须改用匹配并发模型的数据结构或显式锁

---

<!-- arc: ARC-202606241146 --> 1 条已归档 (L_EXP-3) (2026-06-24) → ./changes/ARC-202606241146/proposal.md
