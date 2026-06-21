## Why

近期修复已暴露 TX 唤醒遗漏、waker 注册竞态和 busy-poll，但审计确认同源的所有权、等待和寄存器状态问题仍存在，最严重时会造成 Rust UB、永久挂起或访问错误寄存器。继续做性能优化前，必须先把这些正确性约束固化为可验证的 API 与状态模型。

## What Changes

- 重构 RX/TX ring endpoint 所有权，保证底层 SPSC `Reader`/`Writer` 不会被并发别名化，同时保留安全的多 writer 使用能力。
- 为 ring 与 IRQ 等待统一采用 register → recheck → Pending 协议，消除 TX 判空竞态和 ISR 唤醒窗口。
- 使 `embedded-io-async` 的非空 read/write 在暂不可用时等待，并使 flush 等待 TX ring、FIFO 和 shift register 全部排空。
- 将 RX/TX/DRAIN waker、IER cache 和寄存器 accessor 改为 per-port 状态，支持多个 UART 实例互不串扰。
- 让 ISR 保留 backend 的 stride 与架构访问语义，串行化 IER 状态转换，并清除所有允许启用的中断源。
- 使 `init()`、波特率计算和 `test_loopback()` 在无效输入或失败时返回错误并恢复临时硬件状态。
- 为持续 RX 轮询增加明确调度预算，避免 NAPI 路径长期占用 executor。
- 用真实 poll 路径、独立 ring storage、双端口和失败注入测试替换不能见证实现行为的测试。
- **BREAKING**：收紧公开 ring/driver 字段、重复 copier 启动和 ISR 注册接口；调用方必须通过新的唯一 endpoint 与 per-port IRQ state 初始化路径接入。

## Capabilities

### New Capabilities

- `uart-correctness-invariants`: 定义异步 endpoint 所有权、等待契约、per-port IRQ/IER、寄存器状态恢复、调度公平性及对应测试见证。

### Modified Capabilities

- 无。现有 architecture/learned/references/optimization specs 为项目知识与约束记录，不承担本次产品行为契约。

## BDD Scenario Sketch

### Happy Path

- 单个 RX consumer 与安全共享的多个 TX writer 并发工作时，所有 ring 操作均满足底层并发模型，不产生可变别名。
- 非空 async read 在无数据时挂起，收到数据后返回非零字节数；非空 async write 在 ring 满时挂起，有空间后返回非零字节数。
- flush 仅在 TX ring、硬件 FIFO 和 shift register 均为空时完成。
- 两个 UART 同时收发时，各自 IRQ 只唤醒本端口任务，IER 更新互不影响。
- MMIO stride=1 与 stride=4 使用同一 ISR 行为，并保留目标架构规定的 MMIO 指令。
- 有效配置初始化和 loopback 成功后保持既有行为。

### Sad Path

- producer 在 TX copier 判空与挂起边界 push 时，copier 不丢唤醒。
- RX/TX enable/disable 在不同 CPU 交错时，IER 中不相关位不会丢失。
- ReceiverLineStatus、ModemStatus 或不支持的中断到达时，ISR 不留下无限 pending 的中断源。
- 无效 frequency、baud、prescaler 或不可表示 divisor 返回结构化错误，且 UART 的 DLAB、IER、LCR、MCR 和软件 config 不被部分提交。
- loopback 发送失败或数据不匹配时恢复进入测试前的控制状态。
- 重复启动同一 copier 时返回明确错误或保持幂等，不创建第二个 SPSC endpoint 使用者。

### Edge Cases

- 空 buffer 的 async read/write 立即返回 `Ok(0)`。
- 持续 RX 流量达到预算时，copier 主动让出并保持后续可唤醒。
- 计数器接近整数上限时不影响 ring 安全性和控制流正确性。
- 单端口 StarryOS stride=1 现有接入行为保持一致；需要迁移的仅是初始化/ISR API。

## Impact

- 主要代码：`src/async_/{ring_buffer,driver,isr,device_ops}.rs`、`src/{lib,spec,error}.rs`。
- 公开 API：ring/driver 构造与字段可见性、copier 启动、ISR handler/per-port state、部分错误类型。
- 寄存器位：IER `DATA_READY`、`THR_EMPTY`、`RECEIVER_LINE_STATUS`、`MODEM_STATUS`、`DMA_RX_END`、`DMA_TX_END`；LCR `DLAB`；MCR `LOOP_BACK`；LSR `TRANSMITTER_EMPTY` 及错误位。
- StarryOS：需要同步迁移静态 driver/ring 初始化、IRQ handler 参数和 IER enable callback；RISC-V UART0 的 MMIO 地址与 stride=1 不变。
- 依赖：默认不新增运行时依赖；测试依赖是否增加由 design 阶段评估。
- 回滚：保留变更前构造与 ISR 接线的迁移说明；若整体验证失败，按任务边界回退到当前 API，并保留先行加入的复现测试作为未解决见证。
