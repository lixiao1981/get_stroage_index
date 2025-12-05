# Feature Specification: MDBX PlainState Reader

**Feature Branch**: `001-mdbx-plainstate-reader`
**Created**: 2025-12-05
**Status**: Draft
**Input**: User description: "High-performance Rust library for direct MDBX database access to read Erigon PlainState data"

## Clarifications

### Session 2025-12-05

- Q: When the system encounters corrupted RLP data for an individual account during batch iteration, what should be the default behavior? → A: Log corrupted entry with address, skip it, continue iteration
- Q: What level of logging/observability should the system provide for operations and errors? → A: Structured logs with levels (error/warn/info/debug), human-readable and queryable
- Q: What reliability mechanisms should the system provide for production deployment? → A: Basic: fail immediately on any database error

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Single Account State Lookup (Priority: P1)

As a blockchain indexer developer, I need to query individual account states (balance, nonce, code hash) by address so that I can retrieve current state data without going through slow RPC endpoints.

**Why this priority**: This is the most fundamental operation - retrieving a single account's state. Without this, no other functionality is possible. It demonstrates the core value proposition of direct database access.

**Independent Test**: Can be fully tested by opening a database, querying a known account address, and verifying the returned account data (nonce, balance, code hash) matches expected values. Delivers immediate value for single-account lookups.

**Acceptance Scenarios**:

1. **Given** an Erigon database with existing account data, **When** I query an existing Externally Owned Account (EOA) address, **Then** the system returns the account's nonce and balance
2. **Given** an Erigon database with contract accounts, **When** I query a contract address, **Then** the system returns nonce, balance, storage root, code hash, and incarnation data
3. **Given** an Erigon database, **When** I query a non-existent account address, **Then** the system returns None/null without error
4. **Given** an account with zero balance and zero nonce, **When** I query it, **Then** the system correctly handles field omissions in the RLP encoding

---

### User Story 2 - Bulk Account Iteration (Priority: P2)

As a blockchain analytics developer, I need to iterate through all accounts in the database in batches so that I can build complete state snapshots or indices without overwhelming memory.

**Why this priority**: Once single-account lookup works, the next critical need is processing many accounts efficiently. This enables indexing and analytics use cases that process millions of accounts.

**Independent Test**: Can be tested by iterating through a database with known account count, verifying batch sizes are respected, and confirming all accounts are returned exactly once. Delivers value for batch processing and state export scenarios.

**Acceptance Scenarios**:

1. **Given** an Erigon database with 1 million accounts, **When** I iterate with batch size of 1000, **Then** accounts are returned in consistent batches without duplicates or omissions
2. **Given** an in-progress iteration, **When** I pause and resume from a checkpoint address, **Then** iteration continues from the correct position without re-processing
3. **Given** a database transaction held during iteration, **When** the iteration exceeds configured timeout (5 seconds), **Then** the transaction is automatically released to prevent database bloat
4. **Given** multiple concurrent read operations, **When** iterating accounts, **Then** each operation maintains independent transaction isolation

---

### User Story 3 - Contract Storage Slot Access (Priority: P3)

As a DeFi analytics developer, I need to read specific storage slots for contract addresses so that I can analyze contract state variables (e.g., token balances in ERC-20 contracts, vault positions).

**Why this priority**: After basic account access, contract storage access unlocks deeper on-chain analytics. While valuable, it's not needed for basic account indexing, making it lower priority than P1/P2.

**Independent Test**: Can be tested by querying known storage slots for deployed contracts and verifying returned values match expected state. Delivers value for smart contract state analysis independently of account iteration.

**Acceptance Scenarios**:

1. **Given** a contract account with storage data, **When** I query an existing storage slot by 256-bit key, **Then** the system returns the current 256-bit value
2. **Given** a contract account, **When** I query a non-existent storage slot, **Then** the system returns zero/None without error
3. **Given** Erigon's DupSort storage layout (account + slot composite keys), **When** querying storage, **Then** the system correctly navigates the duplicate key structure
4. **Given** multiple storage slots for the same contract, **When** querying them, **Then** each query completes independently with correct values

---

### User Story 4 - Database Version Detection (Priority: P4)

As a tool maintainer, I need to detect the Erigon database version and schema format so that I can fail gracefully or adapt to different Erigon versions rather than silently producing incorrect data.

**Why this priority**: Critical for production reliability but not needed for initial development/testing with a single known database version. Can be added after core functionality is proven.

**Independent Test**: Can be tested by opening databases from different Erigon versions and verifying correct version detection and appropriate warnings/errors. Delivers value for production deployment safety.

**Acceptance Scenarios**:

1. **Given** an Erigon database with version metadata, **When** opening the database, **Then** the system reads and reports the database schema version
2. **Given** a database version incompatible with the reader, **When** attempting operations, **Then** the system immediately errors with clear version mismatch message
3. **Given** a database missing version information, **When** opening it, **Then** the system warns the user and proceeds with best-effort compatibility mode

---

### Edge Cases

- Database opened by Erigon in read-write mode during read-only access attempt: System fails immediately with clear error message indicating lock conflict
- Corrupted RLP data for individual accounts (malformed encoding): System logs the address and error details, skips the corrupted entry, and continues processing remaining accounts
- Database directory doesn't exist or has incorrect permissions: System fails immediately with clear error message indicating the specific issue
- How does the system behave when the database contains accounts with all-zero values (edge case for field omission)?
- What happens during concurrent multi-threaded reads from the same database environment?
- How does the system handle extremely large account balances (values near U256::MAX)?
- What happens when reading from a database that is mid-sync and has incomplete/uncommitted state?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST open an existing Erigon MDBX database in read-only mode from a specified filesystem path
- **FR-002**: System MUST read account state data by 20-byte Ethereum address from the PlainState bucket
- **FR-003**: System MUST decode RLP-encoded account data including nonce, balance, storage root, code hash, and incarnation fields
- **FR-004**: System MUST handle Erigon's field omission optimization where zero values may be omitted from RLP encoding
- **FR-005**: System MUST distinguish between Externally Owned Accounts (EOA) and contract accounts based on presence of code hash
- **FR-006**: System MUST support batch iteration over all accounts with configurable batch size
- **FR-007**: System MUST support checkpoint-based resumption of batch iteration from a specific address
- **FR-008**: System MUST automatically release read transactions after a configurable timeout (default 5 seconds) to prevent database page bloat
- **FR-009**: System MUST support concurrent read-only transactions from multiple threads without blocking
- **FR-010**: System MUST read contract storage slots using Erigon's DupSort layout (composite keys of address + storage key)
- **FR-011**: System MUST detect database version information and warn or error on incompatible schemas
- **FR-012**: System MUST handle database open failures (missing files, permission errors) with clear error messages
- **FR-013**: System MUST handle RLP decoding errors during batch iteration by logging the corrupted entry with its address, skipping the entry, and continuing iteration with remaining accounts
- **FR-014**: System MUST provide thread-safe access to the database environment for concurrent operations
- **FR-015**: System MUST support querying non-existent accounts and return None/null rather than error
- **FR-016**: System MUST provide structured logging with configurable levels (error, warn, info, debug) in both human-readable and machine-queryable formats for all database operations, errors, and significant events
- **FR-017**: System MUST fail immediately and return errors for all database-level failures (connection errors, transaction failures, lock timeouts) without retry or recovery attempts

### Key Entities

- **PlainAccount**: Represents an Ethereum account with fields for nonce (transaction count), balance (wei amount), optional storage root hash (for contracts), optional code hash (for contracts), and optional incarnation number (for contract upgrade tracking). Supports both EOA and contract account types.

- **ErigonDb**: Represents a connection to the MDBX database environment, managing read-only access, transaction lifecycle, and thread-safe concurrent access through shared environment handles.

- **AccountIterator**: Represents a stateful iterator for batch processing of accounts, maintaining checkpoint position, batch size configuration, and transaction management across iteration cycles.

- **StorageSlot**: Represents a contract storage entry with a 256-bit key and 256-bit value, accessed via the DupSort layout structure.

- **DbVersion**: Represents database schema version metadata, including version number and compatibility flags, used to detect Erigon version changes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Single account queries complete in under 1 microsecond average latency
- **SC-002**: System achieves throughput of at least 1,000,000 account reads per second on single CPU core
- **SC-003**: Batch iteration processes at least 1,000,000 accounts per second during sequential scans
- **SC-004**: Performance improvement of at least 100x compared to HTTP JSON-RPC queries (baseline ~6,000 ops/sec)
- **SC-005**: System supports at least 8 concurrent read operations (number of CPU cores) without performance degradation
- **SC-006**: Memory usage during batch iteration stays below 100MB regardless of total account count
- **SC-007**: Full scan of 100 million accounts completes in under 100 seconds
- **SC-008**: Zero memory safety violations detected during testing (no segfaults, use-after-free, data races)
- **SC-009**: System correctly decodes 100% of valid RLP account encodings in test fixtures
- **SC-010**: Database version mismatches are detected and reported before any data reading occurs
- **SC-011**: All error conditions produce structured log entries with sufficient context (operation type, affected address/key, error details) for troubleshooting

### Assumptions

- Erigon database is in a stable, committed state (not mid-sync or corrupted)
- Read-only access is sufficient (no write operations needed)
- Database is on local storage (not network-mounted) for optimal performance
- Target performance metrics assume NVMe SSD storage with sequential I/O optimization
- Erigon version is v2.x series (Erigon 3/Akula compatibility deferred to future versions)
- Users have appropriate filesystem permissions to read the database directory
- Maximum concurrent readers matches CPU core count (typically 8-16 for modern systems)

## Out of Scope

The following are explicitly excluded from this feature:

- Write operations or database modifications (read-only library)
- Historical state queries (AccountHistory/StorageHistory buckets) - deferred to future version
- Block data, transaction data, or receipt data reading
- Merkle proof generation or state root computation
- Async I/O or io_uring integration - deferred to v1.0
- Erigon 3 (Akula) compatibility - deferred to v0.4
- Network-based database access or remote MDBX connections
- Database migration or schema upgrade tools
- GUI or CLI tools (pure library interface only)
