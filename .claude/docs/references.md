# references.md — 外部参考与依赖

> 由 project-rules-generator 初始化，由 project-docs-assistant 日常维护。
> 条目格式: <!-- R{编号} --> 标记开头，支持 grep 精确定位。

---

## 依赖文档

<!-- R1 --> | 依赖 | 版本 | 链接 | 用途 |
<!-- R1 --> | bitflags | 2.x | https://docs.rs/bitflags | 寄存器位字段定义 |
<!-- R2 --> | embedded-io | 0.6 | https://docs.rs/embedded-io/0.6 | 可选，标准 I/O trait 适配 |
<!-- R3 --> | log | 0.4 | https://docs.rs/log | 可选，调试日志 |
<!-- R4 --> | x86_64 | — | https://docs.rs/x86_64 | cfg(x86_64)，PIO inb/outb |

## 16550 规范

<!-- R10 --> | 主题 | 链接 | 说明 |
<!-- R10 --> | 16550 寄存器映射 | https://wiki.osdev.org/Serial_Ports | 偏移/读/写/功能完整表 |
<!-- R11 --> | 波特率除数表 | spec.rs calc_divisor() | frequency/(16*(prescaler+1)*baud_rate) |
<!-- R12 --> | FIFO 触发级别 | spec.rs FifoTriggerLevel | 1/4/8/14 四档 |
<!-- R13 --> | 中断类型解码 | spec.rs InterruptType | ISR bits[3:1] → 7 种中断类型 |

## 领域知识笔记

<!-- R20 --> 16550 UART 核心参数:
- 时钟频率: 1.8432 MHz (标准 PC)
- FIFO 大小: 16 字节 (TX 和 RX 各 16)
- 最大标准波特率: 115200 (除数=1, 无预分频)
- 寄存器间距: x86 PIO=1, ARM SoC 通常=4

<!-- R21 --> RISC-V QEMU virt 机器 UART:
- 0x10000000 起始地址, MMIO, stride=1
- 兼容 16550 寄存器布局
- 中断号: UART0=10 (PLIC source 10)

<!-- R22 --> 中断处理最佳实践:
1. 读 ISR 获取中断类型
2. 按 InterruptType 分发处理
3. ReceivedDataReady: 批量读 RHR 直到 LSR::DATA_READY 清除
4. THR_EMPTY: 写入下一批数据到 THR
5. ReceiverLineStatus: 读 LSR 确认错误类型
6. 处理完毕后 ISR 应为 0b0001(无挂起中断)
