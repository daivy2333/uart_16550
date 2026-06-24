## Context

审计发现异步 ring wrapper 依赖未被类型保证的 SPSC 前提，async trait 没有真正等待，IRQ 使用全局 waker 与非原子 IER cache，ISR 又绕过 backend 直接按 stride=1 访问 MMIO。同步侧的 init 与 loopback 也会在错误路径留下临时寄存器状态。

该变更跨越 async 四个模块及同步配置/错误模型，并要求 StarryOS 迁移 `uart_init.rs` 中的 ring 构造、`CACHED_IER`、enable callback 和 raw-base ISR wrapper。约束包括 `no_std`、MSRV 1.85.1、公开项文档、unsafe 不变量可证明以及不新增运行时依赖。

## Goals / Non-Goals

**Goals:**

- 消除底层 Reader/Writer 并发别名和重复 copier 使用造成的 UB 风险。
- 使 async read/write/flush、ring wait 和 IRQ wait 满足可验证的等待协议。
- 将 IRQ/waker/IER 状态按端口隔离，并统一通过 backend-aware UartPort 访问硬件。
- 使 init、计算 API 与 loopback 的错误路径可恢复、无 panic。
- 建立真实竞态路径、双端口、stride 与失败注入测试。

**Non-Goals:**

- 不实现 DMA 数据传输，只防止未消费 DMA 中断形成风暴。
- 不改变 UART0 地址、波特率默认值、FIFO 大小或 StarryOS TTY 上层语义。
- 不处理审计之外的 CTS 流控策略、LTO 或性能优化 O8。
- 不新增 async runtime、allocator 或第三方并发队列依赖。

## Decisions

### D1：底层 endpoint 单例 + 可注入 RawMutex

每个 embassy `Reader`/`Writer` 只在 ring 初始化时创建一次，并放入私有共享状态。可能被多个调用者访问的 endpoint 使用 `embassy_sync::blocking_mutex::Mutex<M, RefCell<_>>` 串行化，其中 `M: RawMutex` 由 OS 选择；指标和 waker 与 endpoint 共享。

- RX consumer 通过 `take_reader()` 一次性取得，第二次请求返回 `EndpointTaken`。
- TX writer handle 可 Clone，但每次 push 都经同一 RawMutex，保留 StarryOS 多 writer 能力。
- copier endpoint 保持私有，`AtomicBool::compare_exchange` 保证 RX/TX 各只启动一次。
- driver 的 ring 字段改为私有，通过指标 accessor 暴露观测值。

备选方案：完全禁止 writer Clone。它更简单，但会裁剪当前 TTY 多 producer 能力，因此拒绝。引入 MPMC crate 会扩大依赖与审计面，因此拒绝。继续依赖 unsafe SPSC 注释无法阻止安全 API 制造 UB，因此拒绝。

### D2：按条件拆分 waker set，并统一 register→recheck

RX data、TX data、TX space 和 drain 使用独立 waker set，避免一个条件的 wake 覆盖另一个条件。所有 wait future 按以下顺序执行：

1. 快速检查条件；
2. 注册当前 waker；
3. 重新检查条件；
4. 条件满足则 Ready，否则 Pending。

TX copier 判空也使用同一协议。async write 在 ring 满时等待 TX space；async read 在 ring 空时等待 RX data。空 buffer 直接返回 `Ok(0)`。

备选方案：仅交换现有两行顺序。它无法覆盖判空与 register 之间的窗口，也无法实现 async trait 契约，因此拒绝。

### D3：flush 使用硬件完成条件与协作 yield

flush 先等待 TX ring 清空，再检查 `LSR::TRANSMITTER_EMPTY`。THRE IRQ 可推进大部分状态，但 shift register 排空不保证产生新的边沿，因此为 `OsRuntime` 增加协作式 `yield_now()`，flush 在未 TEMT 时 yield 后重查。RX NAPI 预算耗尽时复用同一原语。

备选方案：把 ring 空定义为 flush 完成。它不能证明中间缓冲内容已到达硬件目的地，与已批准场景冲突，因此拒绝。反复重开 THRE IRQ 会制造中断风暴，因此拒绝。

### D4：per-driver IRQ state 与 UartPort 硬件操作

移除全局 `RX_WAKER`、`TX_WAKER`、`DRAIN_WAKER` 和外置 `CACHED_IER`。`AsyncUartDriver` 拥有 per-port IRQ/waker state，并提供 `handle_irq()`。StarryOS IRQ hook 改为调用具体 driver 实例。

扩展 `UartPort`，增加以下在同一 IRQ-safe 锁中执行的能力：

- 读取并解码 ISR；
- 读取 LSR/MSR 以确认中断与查询 TEMT；
- 原子语义地 `update_ier(set, clear)`，在一次锁定内完成寄存器读改写。

StarryOS 的 `ArceOsUartPort` 继续使用 `SpinNoIrq<Uart16550<MmioBackend>>`，因此寄存器访问自动保留 stride 与 RISC-V/aarch64 backend 语义。`UartPort` 文档明确要求锁在 IRQ 上下文可用且不会被本端口 IRQ 重入。

备选方案：给 `IsrRegisters` 增加 stride。它仍绕过 PIO/aarch64 backend，且无法解决 IER cache 与硬件写入次序，因此拒绝。

### D5：ISR 有界排空并处理所有可达来源

`handle_irq()` 在固定上限内循环读取 ISR，直到 no-pending：

- ReceivedDataReady/ReceptionTimeout：清 `IER::DATA_READY`，wake RX；
- THR empty：清 `IER::THR_EMPTY`，wake TX，并在 TEMT 时 wake drain；
- ReceiverLineStatus：读取 LSR，累计错误指标；
- ModemStatus：读取 MSR；
- DMA RX/TX：无 consumer 时清对应 IER 位并累计 unsupported 指标。

达到循环上限仍 pending 时禁用本次无法确认的中断位并记录 storm 指标，防止 IRQ 活锁。

### D6：先验证、后提交 init

所有波特率参数先使用 checked arithmetic 验证，再触碰硬件。计算 API 使用结构化 error 覆盖零值、prescaler 越界、乘法溢出、非整数结果和 u16 divisor 越界；相应签名变化标记为 breaking。

init 顺序调整为：

```text
纯计算验证
  → 保存 SPR
  → SPR 存在测试并恢复 SPR
  → 禁用 IER
  → 设置 DLAB/DLL/DLM并清 DLAB
  → 设置 LCR/FCR/MCR
  → 提交 config
  → 启用目标 IER
  → 等待 transmitter empty
```

SPR 失败发生在其他配置写入前；纯计算失败不触碰硬件。完成后才更新 `self.config`。

备选方案：在现有 `?` 周围加 DLAB restore。它不能恢复提前写入的 config/IER/SPR，也不能消除计算 panic，因此拒绝。

### D7：loopback 使用 finally 风格恢复

公开 `test_loopback()` 保存 MCR/IER，执行返回 `Result` 的内部测试函数，然后无条件恢复 MCR、IER，并通过当前 config 重建 FCR。恢复完成后再返回原测试结果。该结构避免 guard 长期借用 `&mut self` 与内部收发方法冲突。

备选方案：引入 scope-guard 依赖。现有显式 finally 结构足够且更符合无新增依赖约束，因此拒绝。

### D8：有限 NAPI 预算与饱和指标

RX copier 每轮最多处理固定字节或固定 poll 次数；预算耗尽后调用 `R::yield_now().await`。计数器通过 CAS 饱和到整数最大值，且任何控制判断不得依赖累计指标。

### D9：TDD 见证顺序

执行阶段按风险从低耦合测试基建到架构迁移推进：

1. 独立 ring storage 与可控 waker/IRQ 测试工具；
2. endpoint 并发安全、重复启动和真实 lost-wake RED；
3. async read/write/flush 与 yield RED；
4. per-port IRQ、stride、IER 交错与中断确认 RED；
5. init/calculation/loopback 失败原子性 RED；
6. StarryOS 编译与运行接线迁移。

每项先证明旧实现失败，再做最小 GREEN 修改；禁止先改 API 后补测试。

## Sequence Diagrams

### Async write 与 TX copier

```text
Writer              TX ring             TX copier             UART/IRQ
  | push/满             |                    |                     |
  | register(space)     |                    |                     |
  | recheck ------------>                    |                     |
  | Pending             |                    |                     |
  |                     |<--- pop -----------|                     |
  |<---- wake(space) ---|                    |--- send ----------->|
  | retry/push -------->|--- wake(data) ---->|                     |
  |                     |                    | register(TX IRQ)    |
  |                     |                    | enable + recheck -->|
```

### Per-port IRQ

```text
IRQ hook → driver.handle_irq()
              → UartPort.lock()
              → read ISR through Backend(stride/arch)
              → update IER in same lock
              → clear LSR/MSR source as required
              → unlock
              → wake this driver's RX/TX/DRAIN set
```

### Failure-atomic init

```text
validate config ──fail──> Err, no state change
      |
      v
save/test/restore SPR ──fail──> Err, no later register writes
      |
      v
program UART (no fallible arithmetic remains)
      |
      v
commit self.config + enable IER → Ok
```

## Risks / Trade-offs

- [RawMutex 类型参数扩大公开泛型与 StarryOS 迁移量] → 提供类型别名和完整迁移示例，API 测试固定导出面。
- [同步锁增加 TX push 开销] → 锁内仅执行一次 ring slice 操作；用既有 copier 指标比较迁移前后，不以牺牲安全换性能。
- [UartPort IRQ-safe 契约由外部实现] → 将方法集中为单次 `update_ier`，提供 StarryOS 参考实现和交错测试，不再暴露分离 cache/write callback。
- [flush 等待 TEMT 可能延长调用时间] → 使用协作 yield，不自旋；文档明确 flush 完成语义。
- [ISR 循环上限可能延迟极端突发] → 下次 level IRQ 继续处理，同时 storm 指标提供诊断。
- [breaking error/API 影响调用方] → 在同一变更提供编译错误驱动的迁移清单，并验证 StarryOS。

## Migration Plan

1. 在 uart_16550 内先加入 RED 测试与新内部状态，不删除旧 API。
2. 完成 ring/async 契约后切换 driver 私有字段和构造 API。
3. 完成 per-port IRQ/UartPort 扩展后移除全局 waker、raw-base ISR 和 callback。
4. 同步 StarryOS：增加 RawMutex 类型、扩展 ArceOsUartPort、删除 `CACHED_IER`/enable callback、IRQ hook 调用 `driver.handle_irq()`。
5. 完成同步 init/loopback/error 迁移和调用点修复。
6. 运行 uart_16550 全 feature、多 target、Clippy、Miri（可用时）及 StarryOS 构建/硬件测试。

回滚按任务边界进行：保留 RED 测试，先恢复 StarryOS 旧接线，再恢复 uart_16550 旧公开 API；不得留下新旧 IRQ 状态并存。

## Open Questions

无。BDD 默认假设和 Gate 1 已确定范围、兼容目标及 flush 语义。
