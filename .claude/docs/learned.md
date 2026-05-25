# learned.md — 项目学习记忆

> 由 project-rules-generator 初始化，由 project-docs-assistant 日常维护。
> 条目格式: <!-- L{编号} --> 标记开头，支持 grep 精确定位。

---

## API 路径

<!-- L1 --> | 名称 | 路径 | 用途 | 时间 |
<!-- L1 --> | Uart16550::new_port | src/lib.rs:318 | 创建 x86 PIO 实例 | 2026-05-25 |
<!-- L2 --> | Uart16550::new_mmio | src/lib.rs:351 | 创建 MMIO 实例 | 2026-05-25 |
<!-- L3 --> | Uart16550::init | src/lib.rs:406 | 初始化设备配置 | 2026-05-25 |
<!-- L4 --> | Uart16550Tty::new_port | src/tty.rs:136 | TTY PIO 便捷构造 | 2026-05-25 |
<!-- L5 --> | Uart16550Tty::new_mmio | src/tty.rs:171 | TTY MMIO 便捷构造 | 2026-05-25 |
<!-- L6 --> | Backend trait | src/backend/mod.rs:53 | I/O 后端抽象接口 | 2026-05-25 |
<!-- L7 --> | Config::DEFAULT | src/config.rs:167 | 默认 8-N-1 9600 配置 | 2026-05-25 |

## 文件速查

<!-- L8 --> | 名称 | 路径 | 用途 | 时间 |
<!-- L8 --> | 寄存器定义 | src/spec.rs | 16550 寄存器 bitflags + 常量 | 2026-05-25 |
<!-- L9 --> | 错误类型 | src/error.rs | 所有错误枚举定义 | 2026-05-25 |
<!-- L10 --> | MMIO 后端 | src/backend/mmio.rs | 含 aarch64 内联汇编 | 2026-05-25 |
<!-- L11 --> | PIO 后端 | src/backend/pio.rs | x86 inb/outb 内联汇编 | 2026-05-25 |
<!-- L12 --> | embedded-io | src/embedded_io.rs | Read/Write trait 实现 | 2026-05-25 |

## 踩坑档案

<!-- L13 --> ### [aarch64 MMIO 不可虚拟化指令]
- 症状: LLVM 在 aarch64 上可能发出不可虚拟化的 MMIO 读取指令
- 根因: 标准 volatile read 在某些虚拟化环境下行为异常
- 解: 使用 `ldrb`/`strb` 内联汇编替代 `ptr::read_volatile`（见 src/backend/mmio.rs:arch 模块）

<!-- L14 --> ### [QEMU FIFO 禁用死循环]
- 症状: 设置 `fifo_trigger_level: None` 时 QEMU 永不 drain 数据
- 根因: QEMU 设备模型在非 FIFO 模式下不正确处理 THR drain
- 解: 建议始终启用 FIFO（Config::DEFAULT 使用 FifoTriggerLevel::Fourteen）

## 技巧模式

<!-- L15 --> ### [Sealed Trait 防止外部实现]
- 使用 `mod private { pub trait Sealed {} }` 限制 Backend/RegisterAddress 只能在 crate 内实现
- 保证 API 稳定性，防止不兼容的外部实现

<!-- L16 --> ### [bitflags + 便捷枚举双层设计]
- 寄存器使用 bitflags 提供 ABI 兼容的原始位操作
- 同时提供 WordLength/Parity/FifoTriggerLevel 等便捷枚举，通过 from_raw_bits/to_raw_bits 转换

## 依赖关系图

<!-- L17 --> 核心依赖链:
- lib.rs → config.rs → spec.rs (BaudRate, 常量)
- lib.rs → error.rs → spec.rs (NonIntegerDivisorError)
- lib.rs → backend/mod.rs → backend/mmio.rs, backend/pio.rs
- tty.rs → lib.rs (Uart16550)
- embedded_io.rs → lib.rs (Uart16550) + embedded-io crate

## 待探索

<!-- L18 --> - 硬件测试子项目 (test/) 的启动流程和调试接口用法
<!-- L19 --> - Config::prescaler_division_factor 的实际硬件使用场景