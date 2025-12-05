# erc-mdbx-index Baseline Specification

## 1. Overview

### 1.1 项目概述

erc-mdbx-index是一个基于Rust的高性能库，用于直接读取Erigon以太坊客户端的MDBX数据库中的PlainState数据。通过绕过JSON-RPC接口，实现纳秒级的状态访问。

### 1.2 核心价值主张

| 访问方式 | 延迟 | 吞吐量 | 适用场景 |
|----------|------|--------|----------|
| HTTP JSON-RPC | ~150ms/百万条 | ~6,000 ops/sec | 通用查询 |
| IPC JSON-RPC | ~50ms/百万条 | ~20,000 ops/sec | 本地进程 |
| **MDBX Direct** | **~0.8ms/百万条** | **~1,250,000 ops/sec** | 高频分析 |

### 1.3 目标用户

- 区块链索引器开发者
- 高频交易分析系统
- 链上数据挖掘工具
- 侧车(Sidecar)应用构建者

---

## 2. Technical Architecture

### 2.1 系统架构图

```
┌─────────────────────────────────────────────────────────┐
│                   Application Layer                      │
│  ┌─────────────────┐  ┌─────────────────────────────┐   │
│  │ Balance Indexer │  │ Historical State Retriever  │   │
│  └────────┬────────┘  └──────────────┬──────────────┘   │
│           │                          │                   │
├───────────┴──────────────────────────┴───────────────────┤
│                   Business Logic Layer                   │
│  ┌─────────────────────────────────────────────────────┐ │
│  │                    StateReader                       │ │
│  │  - get_account(address) -> Option<PlainAccount>     │ │
│  │  - iter_accounts() -> Iterator<PlainAccount>        │ │
│  │  - get_storage(address, key) -> Option<U256>        │ │
│  └────────────────────────┬────────────────────────────┘ │
│                           │                              │
├───────────────────────────┴──────────────────────────────┤
│                   Data Model Layer                       │
│  ┌─────────────────────────────────────────────────────┐ │
│  │ PlainAccount { nonce, balance, storage_root,        │ │
│  │                code_hash, incarnation }             │ │
│  │ + Custom RLP Decodable implementation               │ │
│  └────────────────────────┬────────────────────────────┘ │
│                           │                              │
├───────────────────────────┴──────────────────────────────┤
│                Database Abstraction Layer (DAL)          │
│  ┌─────────────────────────────────────────────────────┐ │
│  │                     ErigonDb                         │ │
│  │  - Arc<Environment<NoWriteMap>>                     │ │
│  │  - view<F>(f: F) -> Result<T>                       │ │
│  │  - Thread-safe read-only transactions               │ │
│  └────────────────────────┬────────────────────────────┘ │
│                           │                              │
├───────────────────────────┴──────────────────────────────┤
│                     FFI Boundary                         │
│  ┌─────────────────────────────────────────────────────┐ │
│  │                   libmdbx-rs                         │ │
│  │          (Rust bindings for MDBX C library)         │ │
│  └────────────────────────┬────────────────────────────┘ │
│                           │                              │
├───────────────────────────┴──────────────────────────────┤
│                     Storage Layer                        │
│  ┌─────────────────────────────────────────────────────┐ │
│  │                  MDBX Database                       │ │
│  │  Buckets: PlainState, AccountHistory, Config, etc.  │ │
│  └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 2.2 依赖库选型

| Crate | 版本 | 用途 |
|-------|------|------|
| `libmdbx-rs` | latest | MDBX Rust FFI绑定 |
| `alloy-rlp` | latest | 高性能RLP编解码 |
| `alloy-primitives` | latest | Address, B256, U256类型 |
| `tempfile` | latest | 测试用临时MDBX环境 |
| `rayon` | latest | 并行数据处理 |
| `anyhow` | latest | 错误处理 |
| `ctrlc` | latest | 信号处理 |

---

## 3. Data Model Specification

### 3.1 PlainState键编码

```
Key Format: [u8; 20]
Encoding:   Raw 20-byte Ethereum address
Ordering:   Lexicographical (byte order)
Example:    0x742d35Cc6634C0532925a3b844Bc9e7595f5D123
```

### 3.2 PlainAccount值编码

Erigon采用优化的RLP格式，字段可能被省略：

```rust
pub struct PlainAccount {
    pub nonce: u64,                    // 必需，但0可能以特定单字节表示
    pub balance: U256,                 // 必需，变长编码
    pub storage_root: Option<B256>,    // 可选，空时省略
    pub code_hash: Option<B256>,       // 可选，EOA无此字段
    pub incarnation: Option<u64>,      // 版本相关，合约自毁/重建追踪
}
```

### 3.3 字段省略规则

| 账户类型 | nonce | balance | storage_root | code_hash | incarnation |
|----------|-------|---------|--------------|-----------|-------------|
| 空EOA | 0 | 0 | ✗ | ✗ | ✗ |
| 普通EOA | ✓ | ✓ | ✗ | ✗ | ✗ |
| 新合约 | ✓ | ✓ | ✓/✗ | ✓ | ✓/✗ |
| 重建合约 | ✓ | ✓ | ✓/✗ | ✓ | ✓ |

---

## 4. API Specification

### 4.1 ErigonDb (数据库抽象层)

```rust
impl ErigonDb {
    /// 以只读模式打开Erigon MDBX数据库
    /// 
    /// # Arguments
    /// * `path` - MDBX数据库目录路径
    /// 
    /// # Returns
    /// * `Result<Self>` - 成功返回ErigonDb实例
    /// 
    /// # Errors
    /// * 路径不存在或无权限
    /// * MDBX环境初始化失败
    /// * 数据库版本不兼容
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self>;
    
    /// 在只读事务中执行操作
    /// 
    /// # Arguments
    /// * `f` - 接收事务引用的闭包
    /// 
    /// # Returns
    /// * `Result<T>` - 闭包的返回值
    /// 
    /// # Note
    /// 事务在闭包返回后自动提交释放
    pub fn view<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Transaction<RO>) -> Result<T>;
}
```

### 4.2 StateReader (状态读取器)

```rust
impl StateReader {
    /// 创建状态读取器
    pub fn new(db: ErigonDb) -> Self;
    
    /// 获取单个账户状态
    /// 
    /// # Arguments
    /// * `address` - 20字节以太坊地址
    /// 
    /// # Returns
    /// * `Ok(Some(account))` - 账户存在
    /// * `Ok(None)` - 账户不存在
    /// * `Err(_)` - 读取或解码错误
    pub fn get_account(&self, address: Address) -> Result<Option<PlainAccount>>;
    
    /// 批量迭代所有账户
    /// 
    /// # Arguments
    /// * `batch_size` - 每批处理的账户数量(推荐1000)
    /// 
    /// # Returns
    /// * 账户迭代器，支持断点续传
    pub fn iter_accounts(&self, batch_size: usize) -> AccountIterator;
    
    /// 获取合约存储槽
    /// 
    /// # Arguments
    /// * `address` - 合约地址
    /// * `key` - 256位存储键
    /// 
    /// # Returns
    /// * 存储值(256位)
    pub fn get_storage(&self, address: Address, key: B256) -> Result<Option<U256>>;
    
    /// 获取数据库版本信息
    pub fn get_db_version(&self) -> Result<DbVersion>;
}
```

### 4.3 PlainAccount (账户模型)

```rust
impl PlainAccount {
    /// 判断是否为EOA(外部拥有账户)
    pub fn is_eoa(&self) -> bool {
        self.code_hash.is_none()
    }
    
    /// 判断是否为合约账户
    pub fn is_contract(&self) -> bool {
        self.code_hash.is_some()
    }
    
    /// 判断是否为空账户(可被清除)
    pub fn is_empty(&self) -> bool {
        self.nonce == 0 
            && self.balance == U256::ZERO 
            && self.code_hash.is_none()
    }
}
```

---

## 5. Error Handling

### 5.1 错误类型定义

```rust
#[derive(Debug, thiserror::Error)]
pub enum ErcMdbxError {
    #[error("Database open failed: {0}")]
    DatabaseOpen(#[from] libmdbx::Error),
    
    #[error("RLP decode failed: {0}")]
    RlpDecode(#[from] alloy_rlp::Error),
    
    #[error("Database version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },
    
    #[error("Bucket not found: {0}")]
    BucketNotFound(String),
    
    #[error("Transaction timeout: held for {duration:?}")]
    TransactionTimeout { duration: Duration },
    
    #[error("Corrupted data at key: {key:?}")]
    CorruptedData { key: Vec<u8> },
}
```

### 5.2 错误处理策略

| 错误类型 | 处理策略 | 恢复方式 |
|----------|----------|----------|
| VersionMismatch | Fail Fast | 升级读取器或降级数据库 |
| RlpDecode | 记录并跳过 | 报告损坏记录，继续处理 |
| TransactionTimeout | 警告并释放 | 自动中断长事务 |
| BucketNotFound | Fail Fast | 检查Erigon配置 |

---

## 6. Testing Specification

### 6.1 测试策略矩阵

| 测试层级 | 测试类型 | 覆盖目标 |
|----------|----------|----------|
| Unit | RLP解码测试 | 所有字段组合 |
| Unit | 边界条件测试 | 空账户、最大值 |
| Integration | 数据库读写测试 | MDBX事务正确性 |
| Integration | 并发读取测试 | 多线程安全性 |
| Performance | 基准测试 | 吞吐量、延迟 |

### 6.2 测试夹具规范

```rust
/// 标准EOA测试夹具
const EOA_FIXTURE: &[u8] = &[
    0xc8,       // RLP list prefix (8 bytes payload)
    0x01,       // nonce = 1
    0x87,       // balance prefix (7 bytes)
    0x0d, 0xe0, 0xb6, 0xb3, 0xa7, 0x64, 0x00, // 1 ETH
];

/// 合约账户测试夹具
const CONTRACT_FIXTURE: &[u8] = &[
    // RLP list with nonce, balance, storage_root, code_hash
    // ... 具体字节序列
];
```

### 6.3 TDD周期定义

```
周期一: 基础架构
  [RED]   test_open_nonexistent_db_returns_error
  [RED]   test_get_account_from_empty_db_returns_none
  [GREEN] 实现 ErigonDb::open, StateReader::get_account
  [REFACTOR] 提取环境配置常量

周期二: EOA解码
  [RED]   test_decode_standard_eoa
  [RED]   test_decode_eoa_with_zero_nonce
  [GREEN] 实现 PlainAccount::decode for EOA
  [REFACTOR] 抽离解码模块

周期三: 合约解码
  [RED]   test_decode_contract_with_storage_root
  [RED]   test_decode_contract_with_incarnation
  [GREEN] 扩展解码逻辑处理可选字段
  [REFACTOR] 添加字段检测辅助函数

周期四: 批量操作
  [RED]   test_iterate_accounts_with_batch
  [RED]   test_iterate_respects_short_transaction
  [GREEN] 实现 AccountIterator
  [REFACTOR] 优化游标复用

周期五: 并发安全
  [RED]   test_concurrent_reads_are_isolated
  [RED]   test_environment_is_sync
  [GREEN] 验证 Arc<Environment> 线程安全
  [REFACTOR] 添加并发文档
```

---

## 7. Performance Requirements

### 7.1 基准指标

| 操作 | 目标延迟 | 目标吞吐量 |
|------|----------|------------|
| 单账户查询 | <1μs | >1M ops/sec |
| 批量迭代(1000条) | <1ms | >1M accounts/sec |
| 全量扫描(100M账户) | <100s | >1M accounts/sec |
| 并行扫描(8核) | <15s | >6M accounts/sec |

### 7.2 内存约束

| 指标 | 限制 |
|------|------|
| 单事务最大持续时间 | 5秒 |
| 批量迭代批次大小 | 1000条 |
| 最大并发读事务数 | CPU核心数 |

### 7.3 磁盘I/O优化

- 使用`madvise(MADV_SEQUENTIAL)`进行顺序扫描预读
- 避免长事务导致的页面无法回收
- 支持地址空间分片并行读取

---

## 8. Configuration

### 8.1 环境配置

```rust
pub struct ErigonDbConfig {
    /// 最大打开数据库数(Erigon需要256+)
    pub max_dbs: u32,                    // default: 256
    
    /// 只读模式标志
    pub read_only: bool,                 // default: true
    
    /// 单事务超时(防止数据库膨胀)
    pub transaction_timeout: Duration,   // default: 5s
    
    /// 并发读事务数限制
    pub max_readers: u32,                // default: num_cpus
}
```

### 8.2 Erigon桶名称常量

```rust
pub mod buckets {
    pub const PLAIN_STATE: &str = "PlainState";
    pub const ACCOUNT_HISTORY: &str = "AccountHistory";
    pub const STORAGE_HISTORY: &str = "StorageHistory";
    pub const CONFIG: &str = "Config";
    pub const HEADERS: &str = "Headers";
    pub const DB_VERSION: &str = "DbVersion";
}
```

---

## 9. Future Considerations

### 9.1 路线图

| 版本 | 特性 |
|------|------|
| v0.1 | PlainState基础读取 |
| v0.2 | AccountHistory历史状态回溯 |
| v0.3 | StorageHistory存储槽历史 |
| v0.4 | Erigon 3 (Akula)兼容 |
| v1.0 | io_uring异步I/O集成 |

### 9.2 扩展应用场景

1. **实时账户余额索引器** - 微秒级延迟的余额变更追踪
2. **历史状态回溯服务** - 按需重构任意历史时刻状态
3. **默克尔证明生成器** - 结合StateCommitment生成证明
4. **链上数据分析引擎** - TB级数据的高效处理

---

## 10. References

- [Erigon GitHub Repository](https://github.com/ledgerwatch/erigon)
- [MDBX Documentation](https://erthink.github.io/libmdbx/)
- [Ethereum Yellow Paper - State](https://ethereum.github.io/yellowpaper/paper.pdf)
- [RLP Specification](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/)
- [libmdbx-rs Crate](https://crates.io/crates/libmdbx)
- [alloy-rlp Crate](https://crates.io/crates/alloy-rlp)
