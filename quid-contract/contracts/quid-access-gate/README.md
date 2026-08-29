# Quid Access Gate Contract

A reusable Soroban smart contract for evaluating token and NFT holdings to determine mission eligibility.

## Overview

The `quid-access-gate` contract provides flexible gating rules that can be used to restrict access based on:
- **Token Balance**: Require users to hold a minimum balance of a specific token
- **NFT Ownership**: Require users to own a minimum number of NFTs from a specific collection

## Features

- ✅ Configurable access rules with unique IDs
- ✅ Support for token minimum balance gates
- ✅ Support for NFT ownership gates
- ✅ Multiple rule evaluation (all must pass)
- ✅ Rule activation/deactivation
- ✅ Admin-controlled rule management
- ✅ Comprehensive test coverage

## Usage

### Initialization

```rust
// Initialize the contract with an admin
contract.initialize(admin_address);
```

### Configure Rules

```rust
// Create a token balance rule (require 1000 tokens minimum)
let rule_id = contract.configure_rule(
    GateType::TokenBalance,
    token_address,
    1000
);

// Create an NFT ownership rule (require at least 1 NFT)
let nft_rule_id = contract.configure_rule(
    GateType::NftOwnership,
    nft_collection_address,
    1
);
```

### Check Access

```rust
// Check if a user meets a specific rule
let has_access = contract.check_access(user_address, rule_id);

// Check if a user meets multiple rules
let rule_ids = vec![rule_id1, rule_id2];
let has_access = contract.check_multiple_access(user_address, rule_ids);
```

### Manage Rules

```rust
// Get rule details
let rule = contract.get_rule(rule_id);

// Deactivate a rule
contract.deactivate_rule(rule_id);

// Get total number of rules
let count = contract.get_rule_count();
```

## Integration

This contract can be called from:
- `quid-store` during submission (`submit_feedback`)
- `quid-mission-factory` during mission creation
- Any other contract that needs access control

Example integration:
```rust
// In quid-store or factory contract
let access_gate = AccessGateClient::new(&env, &gate_contract_address);
let has_access = access_gate.check_access(&hunter, &mission.gate_rule_id);

if !has_access {
    return Err(Error::AccessDenied);
}
```

## Data Structures

### GateType
```rust
enum GateType {
    TokenBalance,   // Check minimum token balance
    NftOwnership,   // Check NFT ownership count
}
```

### GateRule
```rust
struct GateRule {
    id: u64,                    // Unique rule identifier
    gate_type: GateType,        // Type of gate check
    token_address: Address,     // Token/NFT contract address
    min_amount: i128,           // Minimum required amount
    active: bool,               // Whether rule is active
    created_at: u64,           // Timestamp
}
```

## Error Codes

| Error | Code | Description |
|-------|------|-------------|
| `AlreadyInitialized` | 1 | Contract already initialized |
| `NotInitialized` | 2 | Contract not initialized |
| `RuleNotFound` | 3 | Rule ID does not exist |
| `InvalidAmount` | 4 | Amount must be positive |
| `RuleLimitReached` | 5 | Maximum rules exceeded (1000) |

## Testing

Run the comprehensive test suite:
```bash
cd quid-contract/contracts/quid-access-gate
cargo test
```

### Test Coverage

- ✅ Initialization
- ✅ Token balance gate (pass/fail)
- ✅ NFT ownership gate (pass/fail)
- ✅ Multiple rule evaluation
- ✅ Rule deactivation
- ✅ Invalid parameter handling
- ✅ Error cases

## Building

```bash
cd quid-contract/contracts/quid-access-gate
cargo build --target wasm32-unknown-unknown --release
```

## License

Same as parent project.
