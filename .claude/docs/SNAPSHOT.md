# SNAPSHOT.md - 项目快照

> Generated at 2026-05-25
> Last updated: 2026-06-21
> 2026-06-21: M4 Sync 已回退到 pre-M4 基线（60c5729），原代码保留在 feat/uart-16550-async-temp。Q15 增量重融合进行中。
> 2026-06-17: 同步 feat/uart-16550-async 分支（Q13 async extraction 完成）

---

## 当前状态

**Phase**: Q15 M4+ 增量重融合
**Status**: 已回退到 pre-M4 基线（60c5729 — OS trait 清理），代码与 StarryOS `04f8920` 对应
**Branch**: feat/uart-16550-async（与 StarryOS 同名分支协同开发）
**文档体系**: OpenSpec (openspec/specs/*) + .claude/docs/{SNAPSHOT,tasks}

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
│   ├── os/                 # [Q13 async] 2 OS 抽象 trait（ADR-036 清理后）
│   │   └── mod.rs           # OsRuntime, OsWakerSet
│   └── async_/             # [Q13 async] 异步 UART 栈（async feature）
│       ├── mod.rs           # 模块入口
│       ├── isr.rs           # 旧版全局 waker（已弃用，保留兼容）→ 新路径: driver.handle_irq()
│       ├── ring_buffer.rs   # RingBufRx/RingBufTx（TX writer 由 RawMutex 保护）
│       ├── driver.rs        # AsyncUartDriver + UartPort trait（per-port waker + handle_irq）
│       └── device_ops.rs    # AsyncUartReader/Writer（register→recheck→Pending 等待）
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
**最近提交**（正确性修复系列 12 commits）:
  - 3c90aff feat(uart-async): NAPI byte budget with cooperative yield (7.x)
  - 308c3fe fix(uart-async): loopback state recovery on all error paths (6.x)
  - e2e50a7 fix(uart-async): checked arithmetic in baud rate calculations (5.x)
  - 0090bbe feat(uart-async): per-port IRQ state with UartPort backend-aware ISR (4.x)
  - 924f500 feat(uart-async): yield_now + TEMT + real flush (3.4-3.5)
  - 8a5e61b fix(uart-async): async waiting for embedded-io-async Read/Write (3.3)
  - db49b98 fix(uart-async): register→recheck→Pending protocol in TX copier (3.2)
  - 76a21c7 test(uart-async): RED tests for lost-wake and async I/O (3.1)
  - 739968a feat(uart-async): take_reader() one-shot gate (2.3)
  - 4023ce6 feat(uart-async): RawMutex to RingBufTx (2.2)
  - 463d897 test(uart-async): RED tests for endpoint ownership (2.1)
  - 144bd4a test(uart-async): isolate test storage per test (1.2)
**测试**: 56 unit + 1 integ + 8 doctest = 65 passed, clippy 0 warnings

---

## 在 StarryOS 中的角色

uart_16550 是 StarryOS 串口子系统的底层驱动模块，Q13 之后提供**完整异步栈**：

- StarryOS (RISC-V) 使用 `Uart16550<MmioBackend>` 实例（`new_mmio(0x10000000, 1)`）
- 启用 `async` feature 后，提供完整异步 UART 栈：
  - ISR handler（AtomicWaker 模式）
  - Ring buffer（embassy SPSC + 批量 push/pop）
  - Copier driver（NAPI coalescing）
  - Device ops（AsyncUartReader/Writer，集成 TtyRead/TtyWrite + embedded-io-async）
- OS 抽象层（2 trait，ADR-036 清理后）：StarryOS 仅需实现 `OsRuntime` / `OsWakerSet`
- Per-port IRQ 状态：waker 从全局 static 迁移到 AsyncUartDriver 实例字段
- Backend-aware ISR：通过 UartPort trait 访问寄存器，保留 stride/架构语义
- TX 安全共享：RingBufTx 使用 RawMutex 保护 writer，支持 Clone-safe AsyncUartWriter

**架构分工**（Q13 完成后）：
```
uart_16550 crate (async feature)
├── 硬件驱动层：Uart16550<MmioBackend>
├── 异步逻辑层：per-port ISR + ring buffer + copier + device_ops
├── OS 抽象层：2 trait (OsRuntime, OsWakerSet)
└── UartPort trait：IRQ-safe 寄存器访问 (update_ier, read_isr, read_lsr, read_msr)
StarryOS kernel
├── 适配层：ArceOsRawMutex + ArceOsRuntime + ArceOsWakerSet + ArceOsUartPort
└── 集成层：初始化 + TTY 绑定 + ISR 接线 (driver.handle_irq())
```

---

## 下一步

- Q6: VisionFive2 真板验证（StarryOS 侧，等待硬件）
- 评估 LTO 重新启用时机（ADR-034）
- M5: ArceOS stdin/stdout/readiness 接入（在 ArceOS 侧进行）
- 归档 OpenSpec 变更 `fix-uart-correctness-invariants`