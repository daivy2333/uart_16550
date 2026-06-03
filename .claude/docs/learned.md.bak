# learned.md — 项目学习记忆

> 由 project-rules-generator 初始化，由 project-docs-assistant 日常维护。
> 条目格式: <!-- L{编号} --> 标记开头，支持 grep 精确定位。

---

## API 路径

<!-- L1 --> | 名称 | 路径 | 用途 | 时间 |
<!-- L1 --> | Uart16550::new_port | src/lib.rs | unsafe fn, 创建 x86 PIO 实例, 参数 base_port: u16 | 2026-05-25 |
<!-- L2 --> | Uart16550::new_mmio | src/lib.rs | unsafe fn, 创建 MMIO 实例, 参数 (NonNull<u8>, stride: u8) | 2026-05-25 |
<!-- L3 --> | Uart16550::init | src/lib.rs | fn init(&mut self, config: Config) -> Result<(), InitError> | 2026-05-25 |
<!-- L4 --> | Uart16550Tty::new_port | src/tty.rs | TTY PIO 便捷构造, 自动 init + loopback test | 2026-05-25 |
<!-- L5 --> | Uart16550Tty::new_mmio | src/tty.rs | TTY MMIO 便捷构造, 自动 init + loopback test | 2026-05-25 |
<!-- L6 --> | Backend trait | src/backend/mod.rs | I/O 后端抽象, sealed, 关联类型 Address | 2026-05-25 |
<!-- L7 --> | Config::DEFAULT | src/config.rs | 默认 8-N-1 9600, FIFO 触发 14, IER::DATA_READY | 2026-05-25 |

### 数据 I/O

<!-- L20 --> | 名称 | 签名 | 说明 |
<!-- L20 --> | try_receive_byte | fn(&mut self) -> Result<u8, ByteReceiveError> | 非阻塞读一字节 |
<!-- L21 --> | try_send_byte | fn(&mut self, byte: u8) -> Result<(), ByteSendError> | 非阻塞写一字节 |
<!-- L22 --> | receive_bytes | fn(&mut self, &mut [u8]) -> usize | 非阻塞读尽可能多字节 |
<!-- L23 --> | send_bytes | fn(&mut self, &[u8]) -> usize | 非阻塞写尽可能多字节 |
<!-- L24 --> | receive_bytes_exact | fn(&mut self, &mut [u8]) | 自旋阻塞直到读满 |
<!-- L25 --> | send_bytes_exact | fn(&mut self, &[u8]) | 自旋阻塞直到写完 |

### 状态查询

<!-- L30 --> | 名称 | 签名 | 说明 |
<!-- L30 --> | ready_to_receive | fn(&mut self) -> Result<(), ByteReceiveError> | LSR::DATA_READY 检查 |
<!-- L31 --> | ready_to_send | fn(&mut self) -> Result<(), ByteSendError> | LSR::THR_EMPTY 检查 |
<!-- L32 --> | test_loopback | fn(&mut self) -> Result<(), LoopbackError> | 环回自检 |
<!-- L33 --> | check_connected | fn(&mut self) -> Result<(), RemoteReadyToReceiveError> | 检查远程连接 |

### 寄存器读取

<!-- L40 --> | 名称 | 返回类型 | 说明 |
<!-- L40 --> | ier() | IER | 中断启用寄存器 |
<!-- L41 --> | isr() | ISR | 中断状态寄存器 |
<!-- L42 --> | lcr() | LCR | 线路控制寄存器 |
<!-- L43 --> | mcr() | MCR | Modem 控制寄存器 |
<!-- L44 --> | lsr() | LSR | 线路状态寄存器 |
<!-- L45 --> | msr() | MSR | Modem 状态寄存器 |
<!-- L46 --> | spr() | SPR | 便笺簿寄存器 |
<!-- L47 --> | dll_dlm() | (DLL, DLM) | 除数锁存器（临时设 DLAB） |
<!-- L48 --> | config_register_dump() | ConfigRegisterDump | 一次读取所有配置寄存器 |

## 文件速查

<!-- L8 --> | 名称 | 路径 | 用途 | 时间 |
<!-- L8 --> | 寄存器定义 | src/spec.rs | 16550 寄存器 bitflags + 常量 + 计算函数 + InterruptType | 2026-05-25 |
<!-- L9 --> | 错误类型 | src/error.rs | InvalidAddressError, InitError, LoopbackError, ByteReceiveError, ByteSendError, RemoteReadyToReceiveError | 2026-05-25 |
<!-- L10 --> | MMIO 后端 | src/backend/mmio.rs | MmioBackend, MmioAddress(NonNull<u8>), aarch64 内联汇编 | 2026-05-25 |
<!-- L11 --> | PIO 后端 | src/backend/pio.rs | PioBackend, PortIoAddress(u16), x86 inb/outb | 2026-05-25 |
<!-- L12 --> | embedded-io | src/embedded_io.rs | Read/Write/WriteReady/ReadReady trait 实现 | 2026-05-25 |
<!-- L13 --> | 配置类型 | src/config.rs | Config struct + BaudRate enum + 默认值 | 2026-05-25 |
<!-- L14 --> | TTY 封装 | src/tty.rs | Uart16550Tty, impl fmt::Write, 自动 \n→\r\n | 2026-05-25 |

## 寄存器速查

<!-- L50 --> | 偏移 | 读 | 写 | DLAB=1 | 说明 |
<!-- L50 --> | 0 | RHR | THR | DLL | 数据/除数低位 |
<!-- L51 --> | 1 | IER | IER | DLM | 中断启用/除数高位 |
<!-- L52 --> | 2 | ISR | FCR | — | 中断状态/FIFO 控制 |
<!-- L53 --> | 3 | LCR | LCR | — | 线路控制（含 DLAB 位） |
<!-- L54 --> | 4 | MCR | MCR | — | Modem 控制 |
<!-- L55 --> | 5 | LSR | — | — | 线路状态（只读） |
<!-- L56 --> | 6 | MSR | — | PSD | Modem 状态/预分频器 |
<!-- L57 --> | 7 | SPR | SPR | — | 便笺簿 |

## 中断速查

<!-- L60 --> ISR IIC 解码表 (bits[3:1]):
<!-- L60 --> | IIC | InterruptType | 优先级 | 清除方式 |
<!-- L60 --> | 0b011 | ReceiverLineStatus | 1 | 读 LSR |
<!-- L61 --> | 0b010 | ReceivedDataReady | 2 | 读 RHR |
<!-- L62 --> | 0b110 | ReceptionTimeout | 2 | 读 RHR |
<!-- L63 --> | 0b001 | TransmitterHoldingRegisterEmpty | 3 | 写数据或读 ISR |
<!-- L64 --> | 0b000 | ModemStatus | 4 | 读 MSR |
<!-- L65 --> | 0b111 | DmaReceptionEndOfTransfer | 5 | 读 ISR（非标准） |
<!-- L66 --> | 0b101 | DmaTransmissionEndOfTransfer | 6 | 读 ISR（非标准） |

<!-- L70 --> IER 中断启用位:
<!-- L70 --> | 位 | 名称 | 说明 |
<!-- L70 --> | 0 | DATA_READY | 数据就绪中断 |
<!-- L71 --> | 1 | THR_EMPTY | THR 空中断 |
<!-- L72 --> | 2 | RECEIVER_LINE_STATUS | 线路状态错误中断 |
<!-- L73 --> | 3 | MODEM_STATUS | Modem 状态变更中断 |
<!-- L74 --> | 6 | DMA_RX_END | DMA 接收结束（非标准） |
<!-- L75 --> | 7 | DMA_TX_END | DMA 传输结束（非标准） |

## FIFO 速查

<!-- L80 --> | FifoTriggerLevel | 原始位 | 触发阈值 | 说明 |
<!-- L80 --> | One | 0b00 | 1 字符 | 每字符中断 |
<!-- L81 --> | Four | 0b01 | 4 字符 | — |
<!-- L82 --> | Eight | 0b10 | 8 字符 | — |
<!-- L83 --> | Fourteen | 0b11 | 14 字符 | 默认/推荐 |

<!-- L85 --> FIFO 关键常量: FIFO_SIZE = 16, CLK_FREQUENCY_HZ = 1_843_200

## Config 字段

<!-- L90 --> | 字段 | 类型 | 默认值 | 说明 |
<!-- L90 --> | interrupts | IER | DATA_READY | 中断启用掩码 |
<!-- L91 --> | frequency | u32 | 1_843_200 | 时钟频率 Hz |
<!-- L92 --> | prescaler_division_factor | Option<u32> | None | 预分频器 |
<!-- L93 --> | fifo_trigger_level | Option<FifoTriggerLevel> | Some(Fourteen) | None=禁用 FIFO |
<!-- L94 --> | baud_rate | BaudRate | Baud9600 | 波特率 |
<!-- L95 --> | data_bits | WordLength | EightBits | 数据位 |
<!-- L96 --> | extra_stop_bits | bool | false | 额外停止位 |
<!-- L97 --> | parity | Parity | Disabled | 校验 |

## 波特率计算

<!-- L100 --> 公式: baud_rate = frequency / (16 * (prescaler + 1) * divisor)
<!-- L101 --> calc_baud_rate(frequency, divisor, prescaler) -> Result<u32, NonIntegerBaudRateError>
<!-- L102 --> calc_frequency(baud_rate, divisor, prescaler) -> u32
<!-- L103 --> calc_divisor(frequency, baud_rate, prescaler) -> Result<u16, NonIntegerDivisorError>

## init() 执行序列

<!-- L110 --> 1. 保存 config → 2. SPR 存在测试(写0x42/0x73回读) → 3. IER=0 禁用中断
<!-- L111 --> → 4. 设波特率(LCR=DLAB, 写DLL+DLM, LCR=0) → 5. 设 LCR(字长+停止位+校验)
<!-- L112 --> → 6. 配置 FCR(FIFO启用/重置/触发级别) → 7. 设 MCR(DTR+RTS+OUT_2_INT_ENABLE)
<!-- L113 --> → 8. 按 config.interrupts 启用中断 → 9. 等待 THR/FIFO/TSR 清空

## 踩坑档案

<!-- L13 --> ### [aarch64 MMIO 不可虚拟化指令]
- 症状: LLVM 在 aarch64 上可能发出不可虚拟化的 MMIO 读取指令
- 根因: 标准 volatile read 在某些虚拟化环境下行为异常
- 解: 使用 `ldrb`/`strb` 内联汇编替代 `ptr::read_volatile`（见 src/backend/mmio.rs:arch 模块）

<!-- L14 --> ### [QEMU FIFO 禁用死循环]
- 症状: 设置 `fifo_trigger_level: None` 时 QEMU 永不 drain 数据
- 根因: QEMU 设备模型在非 FIFO 模式下不正确处理 THR drain
- 解: 建议始终启用 FIFO（Config::DEFAULT 使用 FifoTriggerLevel::Fourteen）

<!-- L15 --> ### [MCR::OUT_2_INT_ENABLE 是 x86 中断路由位]
- 症状: 不设置此位时 UART 中断不触发
- 根因: 典型x86系统中此位将 UART IRQ 线连接到 PIC/APIC
- 解: init() 已自动设置，但自定义初始化流程需注意

<!-- L16 --> ### [RISC-V 只能用 MmioBackend]
- 症状: PioBackend 在 RISC-V 上不可用
- 根因: PioBackend 仅在 cfg(x86/x86_64) 下编译
- 解: RISC-V 上使用 Uart16550::new_mmio(base_addr, stride)

## 技巧模式

<!-- L17 --> ### [Sealed Trait 防止外部实现]
- 使用 `mod private { pub trait Sealed {} }` 限制 Backend/RegisterAddress 只能在 crate 内实现
- 保证 API 稳定性，防止不兼容的外部实现

<!-- L18 --> ### [bitflags + 便捷枚举双层设计]
- 寄存器使用 bitflags 提供 ABI 兼容的原始位操作
- 同时提供 WordLength/Parity/FifoTriggerLevel 等便捷枚举，通过 from_raw_bits/to_raw_bits 转换

<!-- L19 --> ### [中断处理模式]
- 读 ISR → isr.interrupt_type() 匹配 InterruptType → 按类型执行对应操作清除中断
- ReceivedDataReady: 读 RHR; THR_EMPTY: 写数据; ReceiverLineStatus: 读 LSR; ModemStatus: 读 MSR

<!-- L20 --> ### [MMIO stride 参数]
- stride 是寄存器间距字节数，典型值: 1(x86 PC), 4(ARM/SoC 按字对齐)
- RISC-V QEMU virt 机器 UART stride 通常为 1

## 依赖关系图

<!-- L21 --> 核心依赖链:
- lib.rs → config.rs → spec.rs (BaudRate, 常量)
- lib.rs → error.rs → spec.rs (NonIntegerDivisorError)
- lib.rs → backend/mod.rs → backend/mmio.rs, backend/pio.rs
- tty.rs → lib.rs (Uart16550)
- embedded_io.rs → lib.rs (Uart16550) + embedded-io crate

<!-- L22 --> 类型关系:
- Uart16550<B: Backend> { backend: B, base_address: B::Address, config: Config }
- PioBackend: Address=PortIoAddress(u16), stride=1(固定)
- MmioBackend: Address=MmioAddress(NonNull<u8>), stride=用户指定
- Uart16550Tty<B>(Uart16550<B>): impl fmt::Write, 自动 \n→\r\n

## 错误类型速查

<!-- L30 --> | 错误 | 变体 | 触发场景 |
<!-- L30 --> | InvalidAddressError | InvalidBaseAddress / InvalidStride | 构造时地址无效 |
<!-- L31 --> | InitError | DeviceNotPresent / InvalidBaudRate | init() 失败 |
<!-- L32 --> | LoopbackError | SendError / UnexpectedLoopbackByte / UnexpectedLoopbackMsg | 环回测试失败 |
<!-- L33 --> | ByteReceiveError | (无变体, 单值) | 无数据可读 |
<!-- L34 --> | ByteSendError | NoCapacity / RemoteNotClearToSend | FIFO 满 / 远端未就绪 |
<!-- L35 --> | RemoteReadyToReceiveError | NoRemoteConnectedNoDSR / NoRemoteConnectedNoCD / RemoteNotClearToSend | 远端未连接 |

## 待探索

<!-- L40 --> - 硬件测试子项目 (test/) 的启动流程和调试接口用法
<!-- L41 --> - Config::prescaler_division_factor 的实际硬件使用场景
<!-- L42 --> - embedded-io feature 在 StarryOS 中的集成方式
