# erc-mdbx-index Constitution

## Project Identity

**Name:** erc-mdbx-index  
**Purpose:** 基于Rust语言的Erigon客户端PlainState数据层MDBX直接读取库  
**Vision:** 打破区块链节点与数据使用者之间的"RPC墙"，通过将数据访问下沉到文件系统和内存映射层，释放硬件的全部潜力，构建比现有通用工具快几个数量级的专用分析引擎。

## Core Principles

### 1. 内存安全优先 (Memory Safety First)

Rust的内存安全保证是本项目的基石。所有跨越FFI边界的操作必须：
- 明确区分"零拷贝读取"与"所有权拷贝"
- 严格遵守MDBX事务生命周期管理规范
- 确保从事务中读取的数据引用不会逃逸出事务作用域
- 采用"读取即拷贝"策略避免悬垂指针

### 2. 测试驱动开发 (Test-Driven Development)

面对复杂的二进制数据结构，必须构建完整的"模拟与验证"闭环：
- 使用夹具驱动(Fixture-Driven)测试策略
- 通过tempfile创建临时MDBX环境进行单元测试
- 遵循红-绿-重构(Red-Green-Refactor)迭代周期
- 解码逻辑必须独立于数据库环境以便单独测试

### 3. 分层架构 (Layered Architecture)

代码结构遵循清晰的分层模式以确保可维护性和扩展性：
- **数据库抽象层(DAL):** 封装libmdbx复杂性，提供线程安全的环境访问
- **数据模型层(Model):** 定义专门针对Erigon存储布局的Rust结构体
- **业务逻辑层(Service):** 实现具体的桶操作和状态读取

### 4. 性能至上 (Performance First)

直接读取的核心价值在于极致性能：
- 目标吞吐量：>1,000,000 Ops/sec (相比JSON-RPC的~6,000 Ops/sec)
- 采用"短事务"模式避免数据库文件膨胀
- 支持分批游标实现断点续传式批量遍历
- 利用多核CPU通过rayon进行并行处理

### 5. 快速失败 (Fail Fast)

生产环境中必须预设各种故障模式：
- 数据库版本不匹配时立即报错，防止输出错误数据
- 注册信号处理器确保优雅关闭和锁释放
- 明确当前区块高度状态是否已"Commit"再进行读取

## Non-Negotiables

以下原则不可妥协：

1. **绝不在unsafe代码中引入未定义行为** - 所有FFI交互必须经过严格审查
2. **绝不长时间持有读事务** - 防止MDBX页面无法回收导致磁盘膨胀
3. **绝不假设RLP字段完整性** - 必须处理Erigon的字段省略优化机制
4. **绝不忽略版本兼容性** - 必须检测并处理Erigon数据库Schema变更

## Design Philosophy

### 零成本抽象
利用Rust的特性提供高层API而不牺牲底层性能。

### 上下文感知解码
解码器必须根据RLP列表长度和剩余内容推断缺失字段的默认值。

### 并发友好
Environment通过Arc共享，支持多线程并发只读事务。

### 向后兼容
设计时考虑适配未来Erigon版本(如Erigon 3/Akula)的新数据布局。

## Success Metrics

| 指标 | 目标值 |
|------|--------|
| 单核账户读取吞吐量 | ≥1,250,000 ops/sec |
| 相比JSON-RPC性能提升 | ≥100x |
| 测试覆盖率 | ≥90% |
| 内存安全漏洞 | 0 |
| 生产级可靠性 | 99.9% uptime |
