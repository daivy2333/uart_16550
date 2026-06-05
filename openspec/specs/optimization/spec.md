# optimization/spec.md - 优化记录

> Version: 0.6.0
> Last updated: 2026-06-03
> Migrated from: .claude/docs/optimization.md (2026-05-25)

## Purpose

记录项目中发现的优化点和改进方向，持续提升代码质量和性能。每个优化点包含问题描述、当前影响、建议方案和优先级。

## Requirements

### Requirement: 优化点记录

发现的性能瓶颈、代码异味、技术债务 MUST 记录在 spec 中，包含当前影响和建议方案。

#### Scenario: 发现优化机会

- **WHEN** 开发者发现代码中存在性能问题、重复代码、过度复杂设计等
- **THEN** 必须记录到本 spec，包含：问题描述、当前影响、建议方案、优先级
- **AND** 在评估优先级时考虑 StarryOS 集成需求

#### Scenario: 评估优化价值

- **WHEN** 开发者需要决定是否进行某项优化
- **THEN** 可以参考本 spec 中的记录，评估影响范围和收益

### Requirement: 已识别优化点（O1-O8）

优化记录 MUST 包含 O1 至 O8 八项已知优化点，每项含问题描述、当前影响、建议方案。

#### Scenario: O1 - spin_loop 阻塞无超时

- **WHEN** 调用 `send_bytes_exact` / `receive_bytes_exact` 在低波特率或设备不可用时
- **THEN** 当前影响：调用者必须自行确保设备可用，否则可能死锁
- **AND** 建议方案：可考虑添加超时参数版本，或提供 async 版本（需 async runtime）

#### Scenario: O2 - TTY 逐字节发送未利用 FIFO

- **WHEN** 调用 `Uart16550Tty::write_str`
- **THEN** 当前影响：发送效率较低，每次只传一个字节
- **AND** 建议方案：可批量收集字节后通过 `send_bytes` 发送，但需处理换行/退格的特殊字符

#### Scenario: O3 - embedded-io 自旋阻塞

- **WHEN** 在 embedded-io 场景下使用 Read/Write
- **THEN** 当前影响：使用 spin_loop 阻塞等待，可能无限阻塞
- **AND** 建议方案：可考虑实现 embedded-io-async 版本

#### Scenario: O4 - 硬件测试覆盖不足

- **WHEN** 运行 test/ 硬件测试子项目
- **THEN** 当前影响：仅在 x86 目标上运行，MMIO 后端未在真实硬件上充分验证
- **AND** 建议方案：添加 ARM/RISC-V 硬件测试支持

#### Scenario: O5 - DMA 模式缺少使用示例

- **WHEN** 尝试使用 DMA 模式
- **THEN** 当前影响：`FCR::DMA_MODE` 和 `ENABLE_DMA_END` 功能无实际验证
- **AND** 建议方案：添加 DMA 相关的文档和测试场景

#### Scenario: O6 - 批量读写 API（中等优先级）

- **WHEN** copier 任务逐字节调用 `try_receive`/`try_send`
- **THEN** 当前影响：每次都读 LSR 寄存器，寄存器访问开销累积
- **AND** 建议方案：添加批量版本，一次 LSR 检查后连续读/写多字节，减少 MMIO 访问次数
- **AND** 优先级：中 | 触发条件：copier 循环成为性能瓶颈时

#### Scenario: O7 - FIFO 深度硬编码（低优先级）

- **WHEN** 16550 兼容芯片的 FIFO 深度不是 16
- **THEN** 当前影响：硬编码 16 字节 FIFO 深度，某些 16550A 增强版芯片 FIFO 更大
- **AND** 建议方案：Config 中添加 `fifo_depth` 字段，`FifoTriggerLevel` 计算基于实际深度
- **AND** 优先级：低 | 触发条件：上真实硬件发现 FIFO 大于 16 字节时

#### Scenario: O8 - DMA 模式寄存器完整控制（高优先级）

- **WHEN** StarryOS P2 阶段实现 DMA 传输
- **THEN** 当前影响：IER 已有 DMA_RX_END/DMA_TX_END 位，FCR 有 DMA_MODE/ENABLE_DMA_END，但缺少完整的 DMA 传输控制 API
- **AND** 建议方案：添加 DMA 传输启停、地址设置、传输计数等高层 API
- **AND** 优先级：高 | 触发条件：StarryOS P2 阶段实现 DMA 传输时

#### Scenario: O9 - 异步串口 ISR + RingBuffer + Waker 集成（高优先级）

- **WHEN** StarryOS Q6 阶段实现高性能异步串口
- **THEN** 当前影响：uart_16550 API 全部同步（`try_*` 非阻塞 + `*_exact` 自旋 100% CPU），多任务环境不友好，无自然超时
- **AND** 建议方案：在 StarryOS 侧封装 AsyncUart wrapper（路径 B，详见 `.claude/analysis/embassy-integration.md`），三件套：RingBuffer + AtomicWaker + ISR
- **AND** 关联决策：D1（wrapper 位置）+ D2（自研 vs embassy）+ D3（暂不提 PR）
- **AND** 关联 ADR：`architecture/spec.md` Requirement: 异步集成边界
- **AND** 优先级：高 | 触发条件：StarryOS Q6 阶段启动时
- **AND** 预期收益：空闲 CPU 占用从 100% 降至 ~0%，支持自然超时，多任务公平调度

### Requirement: 优化完成追踪

已完成的优化 MUST 在 spec 中记录完成状态，保留历史记录。

#### Scenario: 完成优化

- **WHEN** 开发者完成了某项优化工作
- **THEN** 必须更新本 spec，标记为已完成，记录完成日期和实际效果
- **AND** 可以在 git commit message 中关联 spec 条目编号

### Requirement: 优化优先级管理

优化点 MUST 在 spec 中标注优先级（高/中/低），合理安排优化顺序。

#### Scenario: 规划优化计划

- **WHEN** 开发者制定优化计划时
- **THEN** 可以参考本 spec 的优先级标注：
  - **高优先级**: O9（异步串口集成） + O8（DMA 模式寄存器完整控制）
  - **中优先级**: O6（批量读写 API）
  - **低优先级**: O7（FIFO 深度可配置化）
- **AND** 应优先处理 StarryOS 集成相关的优化（O8、O9）
