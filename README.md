# get_storage_index

高性能 Rust 库，用于直接从 Erigon 的 MDBX 数据库读取以太坊账户状态数据。

## 特性

- ✅ **高性能**: 直接访问 MDBX 数据库，比 JSON-RPC 快 100 倍以上
- ✅ **零依赖 Erigon**: 只读访问，不需要运行 Erigon 节点
- ✅ **类型安全**: 使用 Alloy 的类型系统（Address, U256, FixedBytes）
- ✅ **内存安全**: 纯 Rust 实现，无 unsafe 代码（除数据库绑定）
- ✅ **分层架构**: 模块化设计，易于维护和扩展

## 快速开始

### 安装

```toml
[dependencies]
get_stroage_index = "0.1"
```

### 基本使用

```rust
use get_stroage_index::StateReader;
use alloy_primitives::Address;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 打开 Erigon 数据库
    let reader = StateReader::open("/path/to/erigon/chaindata")?;
    
    // 查询账户
    let address = Address::from_str("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")?;
    
    if let Some(account) = reader.get_account(address)? {
        println!("Nonce:    {}", account.nonce);
        println!("Balance:  {} wei", account.balance);
        println!("Contract: {}", account.is_contract());
    }
    
    Ok(())
}
```

## 架构

项目采用分层架构设计：

```
src/
├── lib.rs          # 公共 API 和模块导出
├── error.rs        # 错误类型定义
├── model.rs        # 领域模型 (PlainAccount, StorageSlot)
├── codec.rs        # RLP 编解码器
├── db.rs           # MDBX 数据库连接层
└── reader.rs       # 高层读取接口
```

### 层次说明

1. **Database Layer** (`db.rs`): MDBX 环境和事务管理
2. **Model Layer** (`model.rs`): 领域实体定义
3. **Codec Layer** (`codec.rs`): RLP 序列化/反序列化
4. **Reader Layer** (`reader.rs`): 面向用户的高层 API
5. **Error Layer** (`error.rs`): 统一错误处理

## API 文档

### `StateReader`

主要的读取接口：

```rust
// 打开数据库
let reader = StateReader::open(path)?;

// 查询完整账户信息
let account = reader.get_account(address)?;

// 查询单个字段
let balance = reader.get_balance(address)?;
let nonce = reader.get_nonce(address)?;

// 检查状态
let is_contract = reader.is_contract(address)?;
let exists = reader.account_exists(address)?;
```

### `PlainAccount`

账户状态数据结构：

```rust
pub struct PlainAccount {
    pub nonce: u64,              // 交易计数
    pub balance: U256,           // 余额（wei）
    pub storage_root: Option<FixedBytes<32>>,  // 存储根（仅合约）
    pub code_hash: Option<FixedBytes<32>>,     // 代码哈希（仅合约）
    pub incarnation: Option<u64>,              // 化身编号（合约升级）
}
```

## 示例

### 查询账户信息

```bash
cargo run --example query_account /path/to/chaindata 0xYourAddress
```

### 批量迭代账户

```bash
# 以 1000 为批次大小迭代账户
cargo run --example batch_iterate /path/to/chaindata 1000
```

### 读取合约存储

```bash
# 读取合约的存储槽
cargo run --example storage_read /path/to/chaindata 0xContractAddress 0
```

### 数据库信息

```bash
# 显示数据库版本和统计信息
cargo run --example db_info /path/to/chaindata
```

## 性能基准测试

```bash
# 运行性能基准测试（需要真实的 Erigon 数据库）
export ERIGON_DB_PATH=/path/to/chaindata
cargo bench
```

## 开发状态

基于规范 `specs/001-mdbx-plainstate-reader/spec.md`：

- [x] 基础架构和模块设计 ✅
- [x] MDBX 数据库连接（使用 heed） ✅
- [x] RLP 解码器（支持 Erigon 字段省略优化） ✅
- [x] 单账户查询（User Story 1 - P1） ✅
- [x] 批量迭代器（User Story 2 - P2） ✅
- [x] 合约存储读取（User Story 3 - P3） ✅
- [x] 数据库版本检测（User Story 4 - P4） ✅
- [x] 错误处理和结构化日志 ✅
- [x] 集成测试和示例程序 ✅
- [x] 单元测试（13 个测试全部通过） ✅
- [x] 性能基准测试框架 ✅

**🎉 所有核心功能已完成！**

## 技术栈

- **数据库**: `heed` 0.20 (LMDB/MDBX 绑定)
- **RLP**: `alloy-rlp` 0.3
- **以太坊类型**: `alloy-primitives` 0.8
- **错误处理**: `thiserror` 1.0, `anyhow` 1.0
- **日志**: `tracing` 0.1

## Modular Design Reference

Comprehensive architectural guidance for the project is located in:

📚 **Main Documentation**: `/src/docs/RUST_MODULAR_DESIGN.md`

### Quick Start Guides
- 🎯 [Constitution Alignment](src/docs/modular-design/constitution-alignment.md) - How architectural patterns align with project principles
- 📦 [Module Templates](src/docs/modular-design/module-templates.md) - Standard mod.rs structure and visibility patterns
- 🧪 [TDD Workflow](src/docs/modular-design/tdd-workflow-guide.md) - Red-Green-Refactor cycle for modular Rust
- 🔌 [Dependency Injection](src/docs/modular-design/dependency-injection-guide.md) - Rust ownership-aware DI strategies

### Architecture & Design
- 📐 [Layer Definitions](src/docs/modular-design/layer-definitions.md) - Database/Model/Codec/Reader layer responsibilities
- 🎨 [Design Patterns](src/docs/modular-design/design-patterns.md) - Builder, NewType, Trait-First patterns
- 🔄 [Refactoring Guide](src/docs/modular-design/refactoring-guide.md) - Fix circular dependencies, glob imports, visibility leaks

### Testing & Quality
- ✅ [Integration Tests](src/docs/modular-design/integration-test-examples.md) - Cross-layer testing with fixtures
- 📋 [Compliance Checklist](src/docs/modular-design/architectural-compliance-checklist.md) - 10-item pre-review validation
- 🔍 [Error/API Review](src/docs/modular-design/error-type-api-review-guide.md) - Layer boundary error handling

### Performance
- ⚡ [Zero-Cost Abstractions](src/docs/modular-design/zero-cost-abstractions-guide.md) - Static vs dynamic dispatch decisions
- 📊 [Performance Considerations](src/docs/modular-design/performance-considerations.md) - Trait dispatch overhead analysis

### Learning & Assessment
- 📝 [Documentation Quiz](src/docs/modular-design/documentation-quiz.md) - 15-minute comprehension validation
- 💡 [Mock PR Examples](src/docs/modular-design/mock-pr-examples.md) - Annotated code review scenarios

For complete documentation index, see [RUST_MODULAR_DESIGN.md](src/docs/RUST_MODULAR_DESIGN.md).