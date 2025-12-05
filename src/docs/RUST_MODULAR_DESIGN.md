# Rust Modular Design Guidelines

## 模块化设计提示词 (Prompt for AI-Assisted Development)

### 基础提示词模板

```
你是一位Rust系统编程专家，正在为erc-mdbx-index项目设计模块化架构。请遵循以下原则：

1. **单一职责**: 每个模块只负责一个明确的功能域
2. **显式依赖**: 通过use语句明确声明所有依赖，禁止glob import (use xxx::*)
3. **接口隔离**: 使用trait定义抽象接口，实现与接口分离
4. **错误边界**: 每个模块定义自己的错误类型，在模块边界转换
5. **可测试性**: 模块设计必须支持单元测试和mock注入

当前任务: [具体任务描述]
```

---

## 1. 项目目录结构规范

```
erc-mdbx-index/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # 库入口，re-export公共API
│   ├── error.rs               # 统一错误类型定义
│   │
│   ├── db/                    # 数据库抽象层
│   │   ├── mod.rs             # 模块声明和re-export
│   │   ├── environment.rs     # MDBX环境管理
│   │   ├── transaction.rs     # 事务生命周期管理
│   │   └── cursor.rs          # 游标操作封装
│   │
│   ├── model/                 # 数据模型层
│   │   ├── mod.rs
│   │   ├── account.rs         # PlainAccount定义
│   │   ├── storage.rs         # 存储槽模型
│   │   └── types.rs           # 基础类型别名
│   │
│   ├── codec/                 # 编解码层
│   │   ├── mod.rs
│   │   ├── rlp.rs             # RLP编解码实现
│   │   ├── key.rs             # 键编码规则
│   │   └── value.rs           # 值编码规则
│   │
│   ├── reader/                # 业务逻辑层
│   │   ├── mod.rs
│   │   ├── state.rs           # StateReader实现
│   │   ├── history.rs         # 历史状态读取
│   │   └── iterator.rs        # 批量迭代器
│   │
│   └── util/                  # 工具模块
│       ├── mod.rs
│       ├── config.rs          # 配置管理
│       └── signal.rs          # 信号处理
│
├── tests/                     # 集成测试
│   ├── common/                # 测试工具
│   │   └── mod.rs
│   ├── db_test.rs
│   ├── codec_test.rs
│   └── reader_test.rs
│
└── benches/                   # 性能基准
    └── throughput.rs
```

---

## 2. 模块依赖规则

### 2.1 依赖方向图

```
┌─────────────────────────────────────────────────────┐
│                     lib.rs                          │
│              (public API facade)                    │
└─────────────────────┬───────────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
   ┌─────────┐  ┌──────────┐  ┌──────────┐
   │ reader/ │  │  model/  │  │  error   │
   └────┬────┘  └────┬─────┘  └────┬─────┘
        │            │             │
        ▼            ▼             │
   ┌─────────┐  ┌──────────┐       │
   │   db/   │◄─┤  codec/  │       │
   └────┬────┘  └──────────┘       │
        │                          │
        └──────────────────────────┘
              所有模块可依赖error
```

### 2.2 禁止的依赖关系

```rust
// ❌ 禁止: 底层模块依赖上层
// db/environment.rs
use crate::reader::StateReader;  // 错误！

// ❌ 禁止: 循环依赖
// model/account.rs
use crate::codec::rlp::decode_account;  // 错误！codec应依赖model

// ❌ 禁止: 跨层直接依赖
// reader/state.rs
use libmdbx::Transaction;  // 错误！应通过db层抽象
```

### 2.3 正确的依赖示例

```rust
// ✅ 正确: 上层依赖下层
// reader/state.rs
use crate::db::ErigonDb;
use crate::model::PlainAccount;
use crate::codec::AccountDecoder;
use crate::error::Result;

// ✅ 正确: codec依赖model（解码产出model）
// codec/rlp.rs
use crate::model::PlainAccount;

// ✅ 正确: 通过trait抽象依赖
// reader/state.rs
use crate::db::ReadTransaction;  // trait, not concrete type
```

---

## 3. 模块接口设计模式

### 3.1 Trait-First 设计

```rust
// db/mod.rs - 定义抽象接口

/// 只读事务抽象
pub trait ReadTransaction {
    type Error;
    
    fn get(&self, bucket: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn cursor(&self, bucket: &str) -> Result<impl Cursor, Self::Error>;
}

/// 游标抽象
pub trait Cursor {
    fn seek(&mut self, key: &[u8]) -> Result<Option<(&[u8], &[u8])>>;
    fn next(&mut self) -> Result<Option<(&[u8], &[u8])>>;
}

// db/transaction.rs - 具体实现
pub struct MdbxReadTransaction<'env> {
    inner: libmdbx::Transaction<'env, RO>,
}

impl ReadTransaction for MdbxReadTransaction<'_> {
    // 实现...
}
```

### 3.2 New Type 模式

```rust
// model/types.rs

use alloy_primitives::{Address as AlloyAddress, B256, U256};

/// 以太坊地址（强类型包装）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address(AlloyAddress);

impl Address {
    pub fn as_bytes(&self) -> &[u8; 20] {
        self.0.as_ref()
    }
    
    pub fn from_slice(slice: &[u8]) -> Result<Self, InvalidAddressError> {
        if slice.len() != 20 {
            return Err(InvalidAddressError { len: slice.len() });
        }
        Ok(Self(AlloyAddress::from_slice(slice)))
    }
}

/// 存储键（强类型包装）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageKey(B256);
```

### 3.3 Builder 模式

```rust
// db/environment.rs

pub struct ErigonDbBuilder {
    path: Option<PathBuf>,
    max_dbs: u32,
    max_readers: u32,
    read_only: bool,
}

impl ErigonDbBuilder {
    pub fn new() -> Self {
        Self {
            path: None,
            max_dbs: 256,
            max_readers: 126,
            read_only: true,
        }
    }
    
    pub fn path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.path = Some(path.into());
        self
    }
    
    pub fn max_dbs(mut self, n: u32) -> Self {
        self.max_dbs = n;
        self
    }
    
    pub fn writable(mut self) -> Self {
        self.read_only = false;
        self
    }
    
    pub fn build(self) -> Result<ErigonDb> {
        let path = self.path.ok_or(Error::PathRequired)?;
        // 构建逻辑...
    }
}

// 使用
let db = ErigonDbBuilder::new()
    .path("/data/erigon/chaindata")
    .max_dbs(512)
    .build()?;
```

---

## 4. 错误处理模块化

### 4.1 分层错误定义

```rust
// error.rs - 顶层统一错误

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] DbError),
    
    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
    
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
}

pub type Result<T> = std::result::Result<T, Error>;

// db/error.rs - DB层错误
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Environment open failed: {0}")]
    EnvOpen(#[source] libmdbx::Error),
    
    #[error("Transaction begin failed")]
    TxnBegin(#[source] libmdbx::Error),
    
    #[error("Bucket not found: {name}")]
    BucketNotFound { name: String },
}

// codec/error.rs - 编解码层错误
#[derive(Debug, Error)]
pub enum CodecError {
    #[error("RLP decode failed: {0}")]
    RlpDecode(#[from] alloy_rlp::Error),
    
    #[error("Invalid account format: expected {expected} bytes, got {actual}")]
    InvalidFormat { expected: usize, actual: usize },
}
```

### 4.2 错误转换边界

```rust
// reader/state.rs

impl StateReader {
    pub fn get_account(&self, address: Address) -> crate::Result<Option<PlainAccount>> {
        // DB层错误自动转换为顶层Error::Db
        let bytes = self.db.view(|txn| {
            txn.get(buckets::PLAIN_STATE, address.as_bytes())
        })?;
        
        // Codec层错误自动转换为顶层Error::Codec
        match bytes {
            Some(b) => Ok(Some(PlainAccount::decode(&b)?)),
            None => Ok(None),
        }
    }
}
```

---

## 5. 可测试性设计

### 5.1 依赖注入

```rust
// reader/state.rs

pub struct StateReader<D: Database = ErigonDb> {
    db: D,
}

impl<D: Database> StateReader<D> {
    pub fn new(db: D) -> Self {
        Self { db }
    }
    
    pub fn get_account(&self, address: Address) -> Result<Option<PlainAccount>> {
        // 实现...
    }
}

// 测试中使用Mock
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::MockDatabase;
    
    #[test]
    fn test_get_nonexistent_account() {
        let mock = MockDatabase::new();
        let reader = StateReader::new(mock);
        
        let result = reader.get_account(Address::zero());
        assert!(result.unwrap().is_none());
    }
}
```

### 5.2 测试夹具模块

```rust
// tests/common/mod.rs

use tempfile::TempDir;
use crate::db::ErigonDb;

pub struct TestEnv {
    _dir: TempDir,  // 持有以防止提前删除
    pub db: ErigonDb,
}

impl TestEnv {
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let db = ErigonDbBuilder::new()
            .path(dir.path())
            .writable()
            .build()
            .unwrap();
        Self { _dir: dir, db }
    }
    
    /// 注入测试账户数据
    pub fn seed_account(&self, address: Address, account: &PlainAccount) {
        self.db.write(|txn| {
            txn.put(
                buckets::PLAIN_STATE,
                address.as_bytes(),
                &account.encode(),
            )
        }).unwrap();
    }
}

// tests/reader_test.rs
mod common;
use common::TestEnv;

#[test]
fn test_read_seeded_account() {
    let env = TestEnv::new();
    let account = PlainAccount {
        nonce: 42,
        balance: U256::from(1_000_000),
        ..Default::default()
    };
    
    env.seed_account(Address::zero(), &account);
    
    let reader = StateReader::new(env.db.clone());
    let result = reader.get_account(Address::zero()).unwrap().unwrap();
    
    assert_eq!(result.nonce, 42);
}
```

---

## 6. 模块公开API设计

### 6.1 lib.rs Facade模式

```rust
// src/lib.rs

//! # erc-mdbx-index
//! 
//! 高性能Erigon MDBX直接读取库
//!
//! ## Quick Start
//! 
//! ```rust
//! use erc_mdbx_index::{ErigonDb, StateReader, Address};
//! 
//! let db = ErigonDb::open("/path/to/erigon/chaindata")?;
//! let reader = StateReader::new(db);
//! 
//! let account = reader.get_account(address)?;
//! ```

// 模块声明（私有）
mod db;
mod model;
mod codec;
mod reader;
mod error;
mod util;

// 公开API re-export
pub use db::{ErigonDb, ErigonDbBuilder};
pub use model::{PlainAccount, Address, StorageKey};
pub use reader::{StateReader, AccountIterator};
pub use error::{Error, Result};

// 可选：暴露子模块供高级用户使用
pub mod prelude {
    pub use crate::{ErigonDb, StateReader, PlainAccount, Address, Result};
}

pub mod advanced {
    pub use crate::db::{ReadTransaction, Cursor};
    pub use crate::codec::{AccountDecoder, AccountEncoder};
}
```

### 6.2 模块内部可见性控制

```rust
// db/mod.rs

mod environment;
mod transaction;
mod cursor;
mod error;

// 公开给crate内部使用
pub(crate) use environment::Environment;
pub(crate) use transaction::MdbxReadTransaction;
pub(crate) use cursor::MdbxCursor;

// 公开给外部用户
pub use environment::{ErigonDb, ErigonDbBuilder};
pub use error::DbError;

// 公开trait（供高级用户实现自己的backend）
pub use transaction::ReadTransaction;
pub use cursor::Cursor;
```

---

## 7. 提示词使用示例

### 7.1 新模块设计提示词

```
我需要为erc-mdbx-index项目添加一个新的 `history` 模块，用于读取账户历史状态。

请根据以下约束设计模块：

1. 位置：src/reader/history.rs
2. 依赖：只能依赖 db/, model/, codec/, error 模块
3. 接口：
   - HistoryReader struct
   - get_account_at(address, block_number) -> Result<Option<PlainAccount>>
   - iter_account_changes(address) -> impl Iterator<Item=AccountChange>
4. 需要定义 AccountChange 模型（放在 model/history.rs）
5. 必须支持依赖注入以便测试

请提供：
- 模块结构设计
- 公开API签名
- 关键实现伪代码
- 单元测试示例
```

### 7.2 重构提示词

```
当前 StateReader 直接依赖了 libmdbx::Transaction，违反了分层原则。

请重构以符合模块化设计：

1. 在 db/ 模块定义 ReadTransaction trait
2. StateReader 改为泛型 StateReader<T: ReadTransaction>
3. 保持向后兼容：提供类型别名 type StateReader = StateReader<MdbxReadTransaction>
4. 添加 MockTransaction 用于测试

当前代码：
[粘贴当前代码]

请提供重构后的代码和迁移说明。
```

### 7.3 代码审查提示词

```
请审查以下模块代码是否符合erc-mdbx-index的模块化设计规范：

[粘贴代码]

检查清单：
- [ ] 是否违反依赖方向规则
- [ ] 是否有循环依赖风险
- [ ] 错误类型是否定义在正确的层级
- [ ] 公开API是否最小化
- [ ] 是否支持依赖注入测试
- [ ] 命名是否符合Rust惯例

请指出问题并给出修复建议。
```

---

## 8. 检查清单

### 新模块上线前检查

- [ ] 模块位置符合目录结构规范
- [ ] 依赖方向正确（上层依赖下层）
- [ ] 无循环依赖
- [ ] 错误类型定义在本模块并可转换为顶层Error
- [ ] 使用trait抽象外部依赖
- [ ] pub可见性最小化
- [ ] 提供单元测试
- [ ] 在lib.rs正确re-export公开API
- [ ] 添加模块级文档注释
- [ ] cargo clippy无警告
