# SNAPSHOT.md - 项目快照

> Generated at 2026-05-25
> Last updated: 2026-06-03
> 2026-06-03: 文档体系迁移到 OpenSpec（架构/学习/参考/优化/规则已迁出，保留 SNAPSHOT + tasks）

---

## 当前状态

**Phase**: 稳定维护
**Status**: v0.6.0 已发布，作为 StarryOS 高性能异步串口的底层驱动模块使用
**Branch**: dev/optimize
**文档体系**: OpenSpec (openspec/specs/*) + .claude/docs/{SNAPSHOT,tasks} + 5 个 .md.bak 备份

---

## 项目结构

```
uart_16550/
├── src/
│   ├── lib.rs              # 主入口，Uart16550<B> 核心类型 + 全部方法
│   ├── config.rs           # Config struct, BaudRate enum, 默认配置
│   ├── error.rs            # 所有错误类型定义
│   ├── spec.rs             # 16550 规范常量 + 寄存器 bitflags + 计算函数
│   ├── tty.rs              # Uart16550Tty 便捷封装, impl fmt::Write
│   ├── embedded_io.rs      # embedded-io Read/Write trait 实现
│   └── backend/
│       ├── mod.rs           # Backend trait, RegisterAddress trait (sealed)
│       ├── mmio.rs          # MmioBackend (NonNull<u8>, 用户指定 stride)
│       └── pio.rs           # PioBackend (u16 端口, stride=1 固定)
├── tests/
│   └── api.rs              # 编译期 API 类型可见性测试
├── test/                    # QEMU i386 硬件测试子项目
├── Cargo.toml              # v0.6.0, edition 2024, no_std
├── openspec/               # OpenSpec 文档体系（v0.6.0 起）
│   ├── config.yaml         # spec-driven schema + Rust 上下文
│   ├── specs/              # 4 个 domain spec
│   │   ├── architecture/   # 6 项 ADR
│   │   ├── learned/        # API/文件/寄存器/中断/Config/波特率/init 速查
│   │   ├── references/     # 依赖 + 16550 规范 + 领域知识
│   │   └── optimization/   # O1-O8 优化点
│   └── changes/            # 变更提案
├── .claude/docs/           # 状态文档 + 备份
│   ├── SNAPSHOT.md         # 项目状态快照（保留）
│   ├── tasks.md            # 任务追踪（保留）
│   └── *.md.bak            # 5 个旧文档备份
└── .codegraph/             # CodeGraph 索引（18 files, 417 nodes）
```

---

## 技术栈

| 类别 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust (no_std) | edition 2024, MSRV 1.85.1 |
| 核心依赖 | bitflags | 2.x |
| 可选依赖 | embedded-io | 0.6 |

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

**当前分支**: dev/optimize
**最近提交**: 69d05ae - repo(docs)
**未提交更改**: openspec/ (新增), .claude/docs/ (迁移+备份), .codegraph/ (新增)

---

## 在 StarryOS 中的角色

uart_16550 是 StarryOS 串口子系统的底层硬件驱动:
- StarryOS (RISC-V) 使用 `Uart16550<MmioBackend>` 实例
- 通过 `new_mmio(0x10000000 的 NonNull, 1)` 构造
- 异步串口在此基础上添加: 中断驱动、环形缓冲区、零拷贝管道

---

## 下一步

- 配合 StarryOS P0 阶段完成中断驱动串口集成
- 评估 O8 (DMA API) 在 P2 阶段的需求