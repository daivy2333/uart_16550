## 1. 基线与测试基础设施

- [ ] 1.1 记录默认、`async`、`embedded-io` 与 all-features 的当前测试/Clippy/API 基线；依赖：无；验收：`cargo test --no-default-features`、`cargo test --features async` 的完整输出已保存，所有可运行基线为 0 failed。
- [ ] 1.2 将 async 单元测试改为每测试独立 ring storage、runtime 与 waker 状态，移除并行重初始化共享 `static mut`；依赖：1.1；验收：`cargo test --features async` 通过且并行重复运行无共享 fixture。
- [ ] 1.3 建立可控 poll/IRQ/寄存器 mock，能够在 register、enable、recheck、IER 写入和硬件失败边界注入事件；依赖：1.2；验收：mock 自测通过，`cargo test --features async` 0 failed。

## 2. Ring endpoint 所有权与启动约束

- [ ] 2.1 添加 RED 测试：并发/交错 TX writer、重复 RX reader 获取、重复 RX/TX copier 启动和私有字段 API；依赖：1.3；验收：旧实现至少一个目标测试失败，失败原因与 spec 一致。
- [ ] 2.2 引入可注入 `RawMutex` 的私有 endpoint 状态，保证底层 Reader/Writer 单例并安全支持 Clone TX writer；依赖：2.1；验收：2.1 RED 转 GREEN，`cargo test --features async` 通过。
- [ ] 2.3 实现一次性 `take_reader()` 与 copier `compare_exchange` 启动门控，补齐公开错误、文档和 API compile tests；依赖：2.2；验收：重复操作返回明确错误，`cargo test --all-features` 通过。

## 3. 无丢失唤醒与 async I/O 契约

- [ ] 3.1 添加 RED 测试：TX 判空/producer push 交错、RX 空读、TX 满写及空 buffer 行为，测试必须经过真实 future/copier poll 路径；依赖：2.3；验收：旧等待顺序和立即 `Ok(0)` 行为被测试见证为失败。
- [ ] 3.2 拆分 RX data、TX data 与 TX space waker set，并统一实现 register→recheck→Pending；依赖：3.1；验收：lost-wake RED 转 GREEN，`cargo test --features async` 通过。
- [ ] 3.3 实现符合 `embedded-io-async` 契约的 read/write future，非空操作不得暂时返回 `Ok(0)`；依赖：3.2；验收：read_exact/write_all 契约测试通过，`cargo test --all-features` 0 failed。
- [ ] 3.4 添加 flush RED 测试，分别见证 TX ring 非空、THR/FIFO 非空和 `LSR::TRANSMITTER_EMPTY` 未置位；依赖：3.3；验收：旧 no-op flush 对至少一个场景失败。
- [ ] 3.5 为 `OsRuntime` 增加协作式 `yield_now()` 并实现等待 ring+TEMT 的 flush；依赖：3.4；验收：flush RED 转 GREEN，无 spin loop，`cargo test --features async` 通过。

## 4. Per-port IRQ、backend 与 IER

- [ ] 4.1 添加 RED 测试：双 UART waker 隔离、stride=1/4、RX/TX IER 交错、LineStatus、ModemStatus、DMA unsupported 和 IRQ 循环上限；依赖：1.3；验收：当前全局 waker/raw MMIO/load-store 实现被测试见证为失败。
- [ ] 4.2 扩展 `UartPort` 的 backend-aware IRQ/LSR/MSR/IER/TEMT 操作，并把 waker 与 IRQ 指标移入 driver 实例；依赖：4.1、3.5；验收：不存在全局 RX/TX/DRAIN waker 或外置 cache，双端口与 stride 测试通过。
- [ ] 4.3 实现同一 IRQ-safe 锁内的 IER 读改写与有界 ISR 排空，显式确认 Line/Modem 并禁用无 consumer 的 DMA 位；依赖：4.2；验收：IER 交错与全部中断来源测试通过，`cargo test --features async` 0 failed。
- [ ] 4.4 删除 raw-base `IsrRegisters` 和 enable callback 旧路径，更新公开文档、示例与 API tests；依赖：4.3；验收：旧 API 编译失败见证和新 API 编译通过见证均明确，`cargo doc --all-features --no-deps` 通过。

## 5. Checked 计算与失败原子 init

- [ ] 5.1 添加 RED 测试：frequency/baud=0、prescaler 越界、乘法溢出、非整数及 u16 divisor 越界，验证无 panic；依赖：1.3；验收：旧 debug-assert/unwrap/overflow 路径被测试见证。
- [ ] 5.2 使用 checked arithmetic 和结构化错误重写计算 API，更新调用点与 API docs；依赖：5.1；验收：所有边界测试 GREEN，`cargo test --all-features` 通过。
- [ ] 5.3 添加 RED 测试：calc 失败和 SPR 探测失败时 config、DLAB、IER、LCR、MCR、SPR 无部分提交；依赖：5.2；验收：当前 init 顺序被失败注入测试见证。
- [ ] 5.4 将 init 改为纯验证→SPR 保存/恢复→无失败写入→最终 config/IER 提交；依赖：5.3；验收：失败原子性 RED 转 GREEN，现有 MMIO stride=1/4 init 测试继续通过。

## 6. Loopback 状态恢复

- [ ] 6.1 添加 RED 测试：单字节发送失败、回读不匹配和成功路径均检查 MCR/IER/FCR 恢复；依赖：1.3；验收：旧提前返回路径至少一个测试失败。
- [ ] 6.2 抽取内部 loopback body 并在公开入口用 finally 风格无条件恢复状态；依赖：6.1；验收：全部恢复测试 GREEN，`cargo test --all-features` 通过。

## 7. NAPI 公平性与指标溢出

- [ ] 7.1 添加 RED 测试：持续 RX 在预算耗尽时 yield，accepted/dropped/popped/copier 计数达到上限不 panic 且不参与控制判断；依赖：3.5；验收：旧无限 Ready 与普通 fetch_add 行为被见证。
- [ ] 7.2 实现固定 NAPI 预算、协作 yield 与饱和指标更新；依赖：7.1；验收：公平性/溢出 RED 转 GREEN，既有指标测试继续通过。

## 8. StarryOS 迁移与兼容文档

- [ ] 8.1 在 uart_16550 更新 README、CHANGELOG、公开 re-export 和迁移示例，明确 BREAKING 构造、RawMutex、reader 获取、copier 启动和 IRQ API；依赖：2.3、4.4、5.2；验收：`cargo test --all-features` 与 `cargo doc --all-features --no-deps` 通过。
- [ ] 8.2 在 StarryOS `kernel/src/drivers/uart_init.rs` 增加 RawMutex/`yield_now` 适配，扩展 `ArceOsUartPort`，删除 `CACHED_IER`/enable callback，并让 IRQ hook 调用对应 driver；依赖：3.5、4.4、8.1；验收：StarryOS 目标构建通过且 UART0 地址/stride=1 不变。
- [ ] 8.3 更新 StarryOS async TTY 构造以使用一次性 reader 和 Clone-safe writer，并核对所有 uart_16550 调用点；依赖：8.2；验收：CodeGraph callers 无旧 API，StarryOS async TTY 构建测试通过。

## 9. 两阶段审查与最终验证

- [ ] 9.1 执行 spec compliance review，逐条核对 delta spec 场景与实现/测试；依赖：2-8 全部任务；验收：追踪矩阵无 Missing/Simplified，`openspec validate fix-uart-correctness-invariants --strict` 通过。
- [ ] 9.2 执行 code quality review，重点审查 unsafe/RawMutex 不变量、取消安全、锁顺序、IRQ 重入和 public docs；依赖：9.1；验收：无 open Critical/Important，`cargo clippy --all-targets --all-features -- -D warnings` 通过。
- [ ] 9.3 运行 uart_16550 完整验证矩阵；依赖：9.2；验收：`cargo fmt --all -- --check`、默认/async/all-features tests、doctests、docs、多目标 build 全部 exit 0，Miri 可用时 tests exit 0。
- [ ] 9.4 运行 StarryOS 构建和 QEMU/硬件 UART 验证，记录 ring/copier/storm 指标；依赖：9.3；验收：构建 exit 0、stdin/stdout/flush 正常、无 hang/IRQ storm/新增 drop。

## Requirements Traceability Matrix

| Requirement | Task(s) | Coverage | Simplification | Status |
|---|---|---:|---|---|
| Ring endpoint 并发安全 | 1.2-1.3, 2.1-2.3, 8.3 | 100% | None | ✅ |
| 异步 I/O 等待契约 | 3.1-3.5, 8.2 | 100% | None | ✅ |
| 无丢失唤醒协议 | 1.3, 3.1-3.2 | 100% | None | ✅ |
| Per-port IRQ 状态隔离 | 4.1-4.4, 8.2 | 100% | None | ✅ |
| Backend-aware ISR | 4.1-4.4, 8.2 | 100% | None | ✅ |
| IER 状态转换与中断确认 | 4.1-4.4, 8.2 | 100% | None | ✅ |
| 初始化失败原子性 | 5.1-5.4 | 100% | None | ✅ |
| Loopback 状态恢复 | 6.1-6.2 | 100% | None | ✅ |
| RX 调度公平性 | 3.5, 7.1-7.2 | 100% | None | ✅ |
| 指标溢出不影响控制流 | 7.1-7.2 | 100% | None | ✅ |
| 回归测试见证真实路径 | 1.1-1.3, 各 RED task, 9.1-9.4 | 100% | None | ✅ |

## Execution Stop

本计划在 Gate 2 审批后停在 Phase 3 入口。未获用户下一次明确授权前，不执行以上 checkbox、不创建 RED 测试、不修改源码。
