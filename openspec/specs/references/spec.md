# references/spec.md - 外部参考与依赖

> Version: 0.6.0
> Last updated: 2026-06-03
> Migrated from: .claude/docs/references.md (2026-05-25)

## Purpose

记录项目依赖和外部参考资源，确保依赖可追溯，资源可获取。所有项目依赖必须记录版本信息和用途。

## Requirements

### Requirement: 依赖版本锁定

所有项目依赖 MUST 记录版本信息在 spec 中，确保构建可重现。

#### Scenario: 添加新依赖

- **WHEN** 开发者引入新的外部依赖
- **THEN** 必须记录到本 spec，包含：依赖名称、版本、官方链接、用途说明
- **AND** 必须在 `Cargo.toml` 中显式指定版本约束

#### Scenario: 更新依赖版本

- **WHEN** 开发者升级或降级依赖版本
- **THEN** 必须更新本 spec 的版本记录
- **AND** 在 `Cargo.lock` 中检查传递依赖
- **AND** 在 CHANGELOG.md 中标注更新原因

#### Scenario: 当前项目依赖清单

- **WHEN** 开发者需要了解项目依赖
- **THEN** 可以参考以下清单（截至 v0.6.0）：
  - `bitflags` 2.11（[docs.rs/bitflags](https://docs.rs/bitflags)） - 寄存器位字段定义
  - `embedded-io` 0.7（[docs.rs/embedded-io](https://docs.rs/embedded-io)） - 可选，标准 I/O trait 适配
  - `assert2` 0.4.0（dev-dep） - 测试断言
  - `x86_64`（cfg(x86_64)） - PIO inb/outb 内置支持

### Requirement: 16550 规范参考

16550 UART 规范相关的外部资源 MUST 记录在 spec 中。

#### Scenario: 查询 16550 寄存器映射

- **WHEN** 开发者需要完整的 16550 寄存器布局
- **THEN** 可以参考 [wiki.osdev.org/Serial_Ports](https://wiki.osdev.org/Serial_Ports)（偏移/读/写/功能完整表）
- **AND** 可参考 `src/spec.rs` 的 bitflags 定义

#### Scenario: 计算波特率除数

- **WHEN** 开发者需要计算波特率除数
- **THEN** 可以使用 `spec.rs` 中的 `calc_divisor(frequency, baud_rate, prescaler)` 函数
- **AND** 公式：`frequency / (16 * (prescaler + 1) * baud_rate)`

#### Scenario: 配置 FIFO 触发级别

- **WHEN** 开发者需要选择 FIFO 触发级别
- **THEN** 可以使用 `spec.rs` 中的 `FifoTriggerLevel` 枚举（1/4/8/14 四档）

#### Scenario: 解码中断类型

- **WHEN** 开发者需要解码 ISR 中的中断类型
- **THEN** 可以使用 `spec.rs` 中的 `InterruptType` 枚举（ISR bits[3:1] → 7 种中断类型）

### Requirement: 领域知识笔记

16550 UART 和 RISC-V 平台的领域知识 MUST 记录在 spec 中。

#### Scenario: 16550 UART 核心参数

- **WHEN** 开发者需要 16550 核心参数
- **THEN** 应了解：
  - 时钟频率: 1.8432 MHz（标准 PC）
  - FIFO 大小: 16 字节（TX 和 RX 各 16）
  - 最大标准波特率: 115200（除数=1，无预分频）
  - 寄存器间距: x86 PIO=1, ARM SoC 通常=4

#### Scenario: RISC-V QEMU virt 机器 UART 配置

- **WHEN** 开发者在 RISC-V QEMU virt 机器上使用 uart_16550
- **THEN** 应使用以下配置：
  - 起始地址: 0x10000000
  - 访问方式: MMIO
  - stride: 1
  - 兼容性: 标准 16550 寄存器布局
  - 中断号: UART0=10 (PLIC source 10)

#### Scenario: 中断处理最佳实践

- **WHEN** 开发者实现中断处理
- **THEN** 应遵循以下 6 步流程：
  1. 读 ISR 获取中断类型
  2. 按 `InterruptType` 分发处理
  3. `ReceivedDataReady`: 批量读 RHR 直到 `LSR::DATA_READY` 清除
  4. `THR_EMPTY`: 写入下一批数据到 THR
  5. `ReceiverLineStatus`: 读 LSR 确认错误类型
  6. 处理完毕后 ISR 应为 0b0001（无挂起中断）

### Requirement: 项目分析文档索引

深度分析文档 MUST 在 spec 中建立索引，方便查找。

#### Scenario: 查找项目分析文档

- **WHEN** 开发者需要深度分析
- **THEN** 可以在 `.claude/analysis/` 目录查找（由 openspec-explorer 生成）
- **AND** 索引条目应包含：主题、路径、内容概要

#### Scenario: 查找 embassy / 异步集成方案

- **WHEN** 开发者需要了解 embassy / async 集成可行性
- **THEN** 阅读 `.claude/analysis/embassy-integration.md`（三种路径对比 + 实施建议 + 代码模式）
- **AND** 三种路径：A) 库内 embassy feature（不推荐）/ B) StarryOS wrapper（推荐）/ C) embedded-io-async feature（次选）

#### Scenario: 查找异步架构分层与性能分析

- **WHEN** 开发者需要量化性能或设计分层架构
- **THEN** 阅读 `.claude/analysis/async-architecture.md`（4 层架构 + 性能数据 + ISR 流程图 + StarryOS Q6 对接）
- **AND** 关键数据：spin_loop 100% CPU vs 异步 ISR ~0% CPU（空闲时）
