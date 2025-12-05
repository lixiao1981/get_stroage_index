# 故障排查指南

## 问题：数据库中没有找到账户

如果运行 `db_info` 或 `query_account` 时看到 "No accounts found" 错误，请按以下步骤诊断：

### 步骤 1：运行诊断工具

```bash
cargo run --example inspect_tables /path/to/database
```

这个工具会：
1. 列出数据库中所有的表及其记录数
2. 识别是否存在 PlainState 表
3. 如果 PlainState 存在，显示前 5 个账户地址

### 步骤 2：理解可能的原因

#### 原因 A：路径不正确

**症状**：看到 "No standard Erigon tables found"

**解决方案**：
- Erigon 数据库路径通常是 `chaindata` 或包含 `mdbx.dat` 文件的目录
- 正确示例：
  ```bash
  /home/user/erigon/chaindata          # ✓ 正确
  /path/to/erigon/block_sync/chaindata # ✓ 正确
  /path/to/erigon                      # ✗ 错误（太上层）
  ```

#### 原因 B：PlainState 表不存在

**症状**：看到其他表（Headers, Bodies, Receipts）但没有 PlainState

**解决方案**：
Erigon 可能没有启用 PlainState 模式。检查 Erigon 启动参数：
```bash
# 需要的参数
--prune.mode=archive          # 或者不使用 prune
--db.pagesize=16KB            # 推荐配置

# 确认 PlainState 未被禁用
# 不要使用 --prune PlainState
```

#### 原因 C：数据库为空或正在同步

**症状**：PlainState 表存在但记录数为 0

**解决方案**：
- 等待 Erigon 完成初始同步
- 检查 Erigon 日志确认同步进度
- 对于 BSC（BNB Chain），完全同步可能需要数天

### 步骤 3：验证数据库完整性

如果诊断工具显示 PlainState 有记录，尝试：

```bash
# 1. 查看前几个账户
cargo run --example inspect_tables /path/to/chaindata

# 2. 尝试批量迭代
cargo run --example batch_iterate /path/to/chaindata 100

# 3. 查询特定账户（使用诊断工具显示的地址）
cargo run --example query_account /path/to/chaindata 0x<address>
```

## 常见错误消息

### "Database not found"
```
Error: Database not found: /path/to/db
```
**原因**：路径不存在或没有读取权限
**解决**：检查路径和文件权限

### "Permission denied"
```
Error: Permission denied
```
**原因**：当前用户没有读取数据库的权限
**解决**：
```bash
# 检查权限
ls -la /path/to/chaindata

# 添加读取权限（如果需要）
sudo chmod -R a+r /path/to/chaindata
```

### "Config table not found"
```
WARN: Config table not found - version detection not available
```
**原因**：这是正常的，某些 Erigon 版本没有 Config 表
**解决**：可以忽略，不影响功能

## BSC (BNB Chain) 特殊说明

对于 BSC Erigon，常见配置：

```bash
# 典型的 BSC 数据目录结构
/home/user/bsc-erigon/
  ├── block_sync/
  │   └── chaindata/      # ← 使用这个路径
  │       ├── mdbx.dat
  │       └── mdbx.lck
  └── datadir/

# 正确的命令
cargo run --example db_info /home/user/bsc-erigon/block_sync/chaindata
```

## 需要帮助？

如果以上步骤都无法解决问题，请提供以下信息：

1. `inspect_tables` 的完整输出
2. Erigon 版本和启动参数
3. 数据库大小：`du -sh /path/to/chaindata`
4. 同步状态：Erigon 日志中的最新区块高度
