# SNAPSHOT.md - 项目快照

> Generated at 2026-05-25
> Last updated: 2026-06-17
> 2026-06-03: 文档体系迁移到 OpenSpec（架构/学习/参考/优化/规则已迁出，保留 SNAPSHOT + tasks）
> 2026-06-17: 同步 feat/uart-16550-async 分支（Q13 async extraction 完成 + 5 性能优化提交）

---

## 当前状态

**Phase**: 异步特性开发中（Q13 async extraction 已完成，性能优化迭代中）
**Status**: v0.6.0 基础上新增 `async` feature，提供完整异步 UART 栈（ISR + ring buffer + copier + device_ops），作为 StarryOS 高性能异步串口的可复用底层模块
**Branch**: feat/uart-16550-async（领先 main 21 commits）
**文档体系**: OpenSpec (openspec/specs/*) + .claude/docs/{SNAPSHOT,tasks} + 5 个 .md.bak 备份

---

## 项目结构

```
uart_16550/
├── src/
│   ├── lib.rs              # 主入口，Uart16550<B> 核心类型 + 全部方法（async feature gate）
│   ├── config.rs           # Config struct, BaudRate enum, 默认配置
│   ├── error.rs            # 所有错误类型定义
│   ├── spec.rs             # 16550 规范常量 + 寄存器 bitflags + 计算函数
│   ├── tty.rs              # TtyRead/TtyWrite trait 提取（Q13）
│   ├── embedded_io.rs      # embedded-io Read/Write trait 实现
│   ├── backend/
│   │   ├── mod.rs           # Backend trait, RegisterAddress trait (sealed)
│   │   ├── mmio.rs          # MmioBackend (NonNull<u8>, 用户指定 stride)
│   │   └── pio.rs           # PioBackend (u16 端口, stride=1 固定)
│   ├── os/                 # [Q13 async] 5 OS 抽象 trait
│   │   └── mod.rs           # OsRuntime, OsIrq, OsMmio, OsSpinNoIrq, OsWakerSet
│   └── async_/             # [Q13 async] 异步 UART 栈（async feature）
│       ├── mod.rs           # 模块入口
│       ├── isr.rs           # ISR handler + AtomicWaker
│       ├── ring_buffer.rs   # RingBufRx/RingBufTx (embassy SPSC + 批量操作)
│       ├── driver.rs        # AsyncUartDriver + UartPort trait (NAPI copier)
│       └── device_ops.rs    # AsyncUartReader/Writer (TtyRead/TtyWrite + embedded_io_async)
├── tests/
│   └── api.rs              # 编译期 API 类型可见性测试
├── test/                    # QEMU i386 硬件测试子项目
├── Cargo.toml              # v0.6.0, edition 2024, no_std; features: [async, embedded-io]
├── openspec/               # OpenSpec 文档体系（v0.6.0 起）
│   ├── config.yaml         # spec-driven schema + Rust 上下文
│   ├── specs/              # 4 个 domain spec
│   │   ├── architecture/   # 6 项 ADR
│   │   ├── learned/        # API/文件/寄存器/中断/Config/波特率/init 速查
│   │   ├── references/     # 依赖 + 16550 规范 + 领域知识 + 子项目索引
│   │   └── optimization/   # O1-O8 优化点
│   └── changes/            # 变更提案（当前空）
├── .claude/docs/           # 状态文档 + 备份
│   ├── SNAPSHOT.md         # 项目状态快照（保留）
│   ├── tasks.md            # 任务追踪（保留）
│   └── *.md.bak            # 5 个旧文档备份
└── .codegraph/             # CodeGraph 索引
```

---

## 技术栈

| 类别 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust (no_std) | edition 2024, MSRV 1.85.1 |
| 核心依赖 | bitflags | 2.x |
| 可选依赖 | embedded-io | 0.7 |
| 可选依赖（async feature） | embedded-io-async | 0.6.1（default-features = false） |
| 可选依赖（async feature） | embassy-sync / embassy-hal-internal | latest |
| Features | `default = []` / `embedded-io` / `async` | — |

---

## 关键常量

| 常量 | 值 | 说明 |
|------|------|------|
| CLK_FREQUENCY_HZ | 1_843_200 | 标准时钟频率 |
| FIFO_SIZE | 16 | TX/RX FIFO 各 16 字节 |
| NUM_REGISTERS | 8 | 8 个寄存器偏移 (0-7) |

---

## 核心 API 速查

| 类别 | 关键方法 | 说明 |
|------|----------|------|
| 构造 | new_port(base_port) / new_mmio(addr, stride) | unsafe，返回 Result |
| 初始化 | init(config) | 配置设备，含 SPR 存在检测 |
| 读 | try_receive_byte / receive_bytes / receive_bytes_exact | 非阻塞/阻塞 |
| 写 | try_send_byte / send_bytes / send_bytes_exact | 非阻塞/阻塞 |
| 状态 | lsr() / isr() / ier() / ready_to_send / ready_to_receive | 查询寄存器 |
| 中断 | isr().interrupt_type() | 返回 Option<InterruptType> |
| 诊断 | test_loopback / check_connected / config_register_dump | 自检/连接检测 |

---

## Git 状态

**当前分支**: feat/uart-16550-async
**最近提交**: 37f60fb - revert(uart-async): disable LTO during active development (ADR-034)
**领先 main**: 21 commits（Q13 async extraction + 性能优化 + 文档）
**未提交更改**: 无（working tree clean，仅 `.codegraph/daemon.pid`）

---

## 在 StarryOS 中的角色

uart_16550 是 StarryOS 串口子系统的底层驱动模块，Q13 之后提供**完整异步栈**：

- StarryOS (RISC-V) 使用 `Uart16550<MmioBackend>` 实例（`new_mmio(0x10000000, 1)`）
- 启用 `async` feature 后，提供完整异步 UART 栈：
  - ISR handler（AtomicWaker 模式）
  - Ring buffer（embassy SPSC + 批量 push/pop）
  - Copier driver（NAPI coalescing）
  - Device ops（AsyncUartReader/Writer，集成 TtyRead/TtyWrite + embedded-io-async）
- OS 抽象层（5 trait）：StarryOS 仅需实现 `OsRuntime` / `OsIrq` / `OsMmio` / `OsSpinNoIrq` / `OsWakerSet`（~50 行）

**架构分工**（Q13 完成后）：
```
uart_16550 crate (async feature)
├── 硬件驱动层：Uart16550<MmioBackend>
├── 异步逻辑层：ISR + ring buffer + copier + device_ops
└── OS 抽象层：5 trait
StarryOS kernel
├── 适配层：实现 5 OS trait
└── 集成层：初始化 + TTY 绑定
```

---

## 下一步

- Q6 等待硬件（StarryOS 端）
- 评估 LTO 重新启用时机（ADR-034 临时禁用，待稳定后恢复）
- 维护 Q13 异步栈稳定性，跟踪 overhead 优化（inline + batch 已落地）