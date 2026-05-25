# references.md — 外部参考与依赖

> 由 project-rules-generator 初始化，由 project-docs-assistant 日常维护。
> 条目格式: <!-- R{编号} --> 标记开头，支持 grep 精确定位。

---

## 依赖文档

<!-- R1 --> | 依赖 | 版本 | 链接 | 用途 |
<!-- R1 --> | bitflags | 2.11 | https://docs.rs/bitflags | 寄存器位操作 bitflags 宏 |
<!-- R2 --> | embedded-io | 0.7 | https://docs.rs/embedded-io | 嵌入式 Read/Write trait (可选) |
<!-- R3 --> | assert2 | 0.4 | https://docs.rs/assert2 | 测试断言宏 (dev-dependency) |

## 领域知识笔记

<!-- R4 --> 16550 UART 数据手册: https://caro.su/msx/ocm_de1/16550.pdf
- 8 个寄存器偏移量 (0-7)
- FIFO 大小 16 字节
- 标准时钟频率 1.8432 MHz
- 波特率公式: baud = frequency / (16 * divisor * (prescaler + 1))

<!-- R5 --> Linux 串口控制台文档: https://docs.kernel.org/admin-guide/serial-console.html
- 默认波特率 9600
- 8-N-1 传输配置

<!-- R6 --> rust-osdev 社区: https://github.com/rust-osdev
- 此项目为 rust-osdev 组织下的 crate
- 专注 Rust OS 开发生态

<!-- R7 --> aarch64 MMIO 虚拟化问题: https://github.com/rust-lang/rust/issues/131894
- LLVM 在 aarch64 上可能发出不可虚拟化指令
- 需使用显式 ldrb/strb 汇编

<!-- R8 --> 项目仓库: https://github.com/rust-osdev/uart_16550
- 文档发布: https://docs.rs/uart_16550