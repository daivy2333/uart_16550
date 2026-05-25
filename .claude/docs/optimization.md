# optimization.md — 优化记录

> 由 project-rules-generator 初始化，由 project-docs-assistant 日常维护。
> 条目格式: <!-- O{编号} --> - {问题描述}，每条含当前影响、建议方案。

---

<!-- O1 --> - send_bytes_exact / receive_bytes_exact 使用 spin_loop 等待，在低波特率或设备不可用时可能无限阻塞
- 当前影响: 调用者必须自行确保设备可用，否则可能死锁
- 建议方案: 可考虑添加超时参数版本，或提供 async 版本（需 async runtime）

<!-- O2 --> - Uart16550Tty::write_str逐字节发送，未利用 FIFO 批量传输能力
- 当前影响: 发送效率较低，每次只传一个字节
- 建议方案: 可批量收集字节后通过 send_bytes 发送，但需处理换行/退格的特殊字符

<!-- O3 --> - embedded-io 实现中 Read/Write 使用 spin_loop 阻塞等待
- 当前影响: 在 embedded-io 场景下也可能无限阻塞
- 建议方案: 可考虑实现 embedded-io-async 版本

<!-- O4 --> - test/ 硬件测试子项目仅在 x86 目标上运行，缺少其他架构测试
- 当前影响: MMIO 后端未在真实硬件上充分验证
- 建议方案: 添加 ARM/RISC-V 硬件测试支持

<!-- O5 --> - 缺少 DMA 模式的实际使用示例和测试
- 当前影响: FCR::DMA_MODE 和 ENABLE_DMA_END 功能无实际验证
- 建议方案: 添加 DMA 相关的文档和测试场景

<!-- O6 --> - 批量读写 API（try_receive_batch / try_send_batch）
- 当前影响: copier 任务逐字节调用 try_receive/try_send，每次都读 LSR 寄存器，寄存器访问开销累积
- 建议方案: 添加批量版本，一次 LSR 检查后连续读/写多字节，减少 MMIO 访问次数
- 优先级: 中 | 触发条件: copier 循环成为性能瓶颈时

<!-- O7 --> - FIFO 深度可配置化
- 当前影响: 硬编码 16 字节 FIFO 深度，某些 16550 兼容芯片（如 16550A 增强版）FIFO 更大
- 建议方案: Config 中添加 fifo_depth 字段，FifoTriggerLevel 计算基于实际深度
- 优先级: 低 | 触发条件: 上真实硬件发现 FIFO 大于 16 字节时

<!-- O8 --> - DMA 模式寄存器完整控制
- 当前影响: IER 已有 DMA_RX_END/DMA_TX_END 位，FCR 有 DMA_MODE/ENABLE_DMA_END，但缺少完整的 DMA 传输控制 API
- 建议方案: 添加 DMA 传输启停、地址设置、传输计数等高层 API
- 优先级: 高 | 触发条件: StarryOS P2 阶段实现 DMA 传输时