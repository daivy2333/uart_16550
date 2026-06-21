> Version: 0.1.0
> Last updated: 2026-06-20

## ADDED Requirements

### Requirement: Ring endpoint 并发安全

系统 MUST 保证每个底层 SPSC Reader/Writer 只创建一次，并且所有可并发访问均通过类型所有权或显式互斥串行化。

#### Scenario: 多个 TX writer 并发写入

- **WHEN** 多个 `AsyncUartWriter` 在不同任务或 CPU 上同时写入
- **THEN** 系统 MUST 串行化底层 Writer 访问，不得构造并发可变别名
- **AND** 每次写入 MUST 保持字节切片内部顺序

#### Scenario: 创建 RX reader

- **WHEN** 调用方请求 UART 的异步 reader
- **THEN** 系统 MUST 只允许取得一个 RX consumer capability
- **AND** 重复请求 MUST 返回明确错误而不是创建第二个 Reader 使用者

#### Scenario: 重复启动 copier

- **WHEN** 同一 driver 的 RX 或 TX copier 被重复启动
- **THEN** 系统 MUST 拒绝第二次启动或保持可证明的幂等
- **AND** 不得产生第二个底层 endpoint 使用者

### Requirement: 异步 I/O 等待契约

`embedded-io-async` 适配 MUST 遵循非空 read/write 等待和 flush 到达目的地的契约。

#### Scenario: 非空异步读取暂时无数据

- **WHEN** 使用非空 buffer 调用 async read 且 RX ring 为空
- **THEN** future MUST 保持 Pending
- **AND** 数据到达后 MUST 返回大于零的读取长度

#### Scenario: 非空异步写入暂时无空间

- **WHEN** 使用非空 buffer 调用 async write 且 TX ring 已满
- **THEN** future MUST 保持 Pending
- **AND** 空间可用后 MUST 返回大于零的写入长度
- **AND** 不得对非空 buffer 返回 `Ok(0)`

#### Scenario: 空 buffer I/O

- **WHEN** async read 或 write 收到空 buffer
- **THEN** 操作 MUST 立即返回 `Ok(0)`

#### Scenario: 刷新发送数据

- **WHEN** 调用 async flush
- **THEN** future MUST 等待 TX ring 为空
- **AND** MUST 等待 `LSR::TRANSMITTER_EMPTY` 表明 FIFO 与 shift register 均为空后完成

### Requirement: 无丢失唤醒的等待协议

所有由 ring 或 IRQ 条件驱动的 Pending 转换 MUST 使用 register → recheck → Pending 协议。

#### Scenario: TX copier 在空 ring 上挂起

- **WHEN** producer 在 copier 首次判空与挂起边界写入数据
- **THEN** copier MUST 观察到数据或收到 wake
- **AND** 不得在 ring 非空时无限期保持 Pending

#### Scenario: IRQ enable 边界出现硬件事件

- **WHEN** RX/TX 事件在 waker 注册、IER enable 与条件重查之间发生
- **THEN** 对应 future MUST 继续执行
- **AND** 不得依赖未来额外事件修复已丢失的 wake

### Requirement: Per-port IRQ 状态隔离

每个 UART driver MUST 独立拥有 RX、TX、DRAIN waker 与 IRQ 控制状态。

#### Scenario: 两个 UART 同时触发中断

- **WHEN** 两个 UART 实例分别注册 copier 并同时产生 IRQ
- **THEN** 每个 IRQ MUST 只唤醒所属端口的任务
- **AND** 任一端口的注册不得覆盖另一端口的 waker

#### Scenario: 单端口兼容

- **WHEN** StarryOS 使用现有 UART0 MMIO 地址和 stride=1
- **THEN** 收发行为 MUST 与迁移前保持一致
- **AND** 仅初始化与 IRQ 接线 API 需要迁移

### Requirement: Backend-aware ISR 寄存器访问

ISR MUST 通过端口 backend 抽象访问寄存器，不得自行假设 MMIO stride 或目标架构指令。

#### Scenario: MMIO stride 变化

- **WHEN** UART 使用 stride=1 或 stride=4
- **THEN** ISR MUST 访问对应 stride 下的 ISR、IER、LSR 和 MSR 地址

#### Scenario: aarch64 MMIO

- **WHEN** ISR 在 aarch64 MMIO UART 上访问寄存器
- **THEN** 访问 MUST 保留 backend 规定的 `ldrb`/`strb` 语义
- **AND** 不得绕过 backend 直接使用通用 volatile pointer

### Requirement: IER 状态转换与中断确认

IER 更新与硬件写入 MUST 在同一 per-port 串行化边界内完成，ISR MUST 确认或禁用每个可达中断源。

#### Scenario: RX/TX IER 并发更新

- **WHEN** 不同 CPU 交错 enable 或 disable `IER::DATA_READY` 与 `IER::THR_EMPTY`
- **THEN** 最终 IER MUST 保留所有未被明确清除的位
- **AND** cache 状态与硬件寄存器 MUST 一致

#### Scenario: Line status 中断

- **WHEN** ISR 解码到 `IER::RECEIVER_LINE_STATUS` 对应事件
- **THEN** ISR MUST 读取 LSR 确认并清除来源
- **AND** MUST 记录可观测错误状态

#### Scenario: Modem status 中断

- **WHEN** ISR 解码到 `IER::MODEM_STATUS` 对应事件
- **THEN** ISR MUST 读取 MSR 清除来源

#### Scenario: 不支持的 DMA 中断

- **WHEN** ISR 解码到 `IER::DMA_RX_END` 或 `IER::DMA_TX_END` 对应事件且没有 DMA consumer
- **THEN** ISR MUST 禁用对应 IER 位并记录计数
- **AND** 不得持续返回同一 pending source

### Requirement: 初始化失败原子性

配置验证或设备探测失败时，`Uart16550::init()` MUST 返回错误且不得留下部分提交的软件或硬件状态。

#### Scenario: 无效波特率参数

- **WHEN** frequency 或 baud 为零、prescaler 超出范围、算术溢出或 divisor 无法表示为 u16
- **THEN** 计算 API 与 init MUST 返回结构化错误
- **AND** 不得 panic

#### Scenario: divisor 计算失败

- **WHEN** init 的 divisor 计算失败
- **THEN** LCR `DLAB`、IER、MCR 和软件 config MUST 保持调用前状态

#### Scenario: SPR 设备探测

- **WHEN** SPR 写回测试成功或失败
- **THEN** init MUST 尽力恢复原 SPR 值
- **AND** 失败时不得继续修改其他配置寄存器

### Requirement: Loopback 状态恢复

`test_loopback()` MUST 在所有正常错误退出路径恢复测试前的控制状态。

#### Scenario: loopback 发送或比较失败

- **WHEN** 单字节发送失败或回读内容不匹配
- **THEN** MCR `LOOP_BACK` MUST 恢复为调用前状态
- **AND** IER 与按当前 config 可恢复的 FIFO 配置 MUST 恢复

#### Scenario: loopback 成功

- **WHEN** 全部 loopback 检查通过
- **THEN** 恢复后的控制状态 MUST 与进入测试前一致

### Requirement: RX 调度公平性

RX copier MUST 使用有限轮询预算，并在持续有数据时为其他 executor 任务提供调度机会。

#### Scenario: 持续 RX 流量

- **WHEN** UART 在一个预算周期内持续返回数据
- **THEN** copier MUST 在预算耗尽后主动 yield
- **AND** MUST 保持自身可被再次调度且不丢失后续数据通知

### Requirement: 指标溢出不影响控制流

可观测性计数器 MUST 在达到整数上限时饱和或以不参与安全决策的方式处理。

#### Scenario: 指标达到上限

- **WHEN** accepted、dropped、popped 或 copier 指标达到整数上限
- **THEN** ring 所有权、空满判断、唤醒和 IRQ 控制 MUST 保持正确
- **AND** 指标更新不得 panic

### Requirement: 回归测试必须见证真实路径

每项正确性修复 MUST 具有能在旧实现上失败、在新实现上通过的测试见证。

#### Scenario: waker 顺序回归

- **WHEN** 测试在真实 copier poll 路径的 register/enable/recheck 边界注入事件
- **THEN** 错误顺序 MUST 使测试失败
- **AND** 正确顺序 MUST 使测试通过

#### Scenario: 测试隔离

- **WHEN** ring 和 driver 单元测试并行执行
- **THEN** 每个测试 MUST 使用独立 storage 和状态
- **AND** 不得并发重初始化共享 `static mut` ring

#### Scenario: 跨平台与多端口覆盖

- **WHEN** 执行正确性测试矩阵
- **THEN** MUST 覆盖双端口、stride=1、stride=4、IER 交错、init/loopback 失败注入和 async I/O 契约
