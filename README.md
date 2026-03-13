# Miden Testnet Smart Contracts

A collection of Zero-Knowledge smart contracts built and deployed on the [Miden blockchain](https://miden.xyz) testnet.
Written in MASM (Miden Assembly) and deployed using miden-client v0.13.0.

> 7 contracts deployed on a live ZK testnet — from a simple counter to a multi-sig wallet and token auction.

---

## Contracts

| Contract | Testnet ID | Description |
|---|---|---|
| Counter v2 | mtst1arl70qcqx5m9gqpwmv7kavvpmu6ygdcs | increment, decrement, reset |
| Counter B | mtst1az3vy7vpvwngzqq0g9cp47dxqypfh2lt | independent instance |
| Voting | mtst1azsrn5xxfqx4zqrwtxyqj55v0sjh6lgc | yes / no / abstain |
| Vending Machine | mtst1ar6mf7x8wkutyqpj82krr09975l5f84f | deposit and dispense |
| Escrow | mtst1aqpf5ua452gpyqqc6642lz8zs5zpgjz4 | lock, approve, claim |
| Leaderboard | mtst1aqa59jy2lvzqzqrdhknsgxlc5cwa5f0j | only updates on new high score |
| Multi-sig Wallet | mtst1apmdv03sy3h3qqre4j2mvq0v8uxp34fy | 2-of-3 threshold |
| Token Auction | mtst1apmgekrr6a64uqp9zz5durhm4g4c2rfj | highest bid wins |

---

## Project Structure

```
my-miden-client/
├── masm/
│   ├── accounts/
│   │   ├── counter.masm
│   │   ├── voting.masm
│   │   ├── vending.masm
│   │   ├── escrow.masm
│   │   ├── leaderboard.masm
│   │   ├── multisig.masm
│   │   └── auction.masm
│   └── scripts/
├── src/
│   └── main.rs
└── Cargo.toml
```

---

## Setup

### 1. Install Rust
```bash
curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Clone and build
```bash
git clone https://github.com/lifeofagct/miden-contracts.git
cd miden-contracts
cargo build
```

### 3. Run
```bash
cargo run
```

---

## Key MASM Patterns

### Basic storage (counter)
```
pub proc increment
    push.COUNTER[0..2] exec.active_account::get_item
    add.1
    push.COUNTER[0..2] exec.native_account::set_item
    exec.sys::truncate_stack
end
```

### Conditional execution (escrow)
```
pub proc claim
    push.APPROVED[0..2] exec.active_account::get_item
    drop drop drop
    push.1 eq
    if.true
        push.CLAIMED[0..2] exec.active_account::get_item
        add.1
        push.CLAIMED[0..2] exec.native_account::set_item
    end
    exec.sys::truncate_stack
end
```

---

## Resources

- [Miden Docs](https://docs.miden.xyz)
- [Miden GitHub](https://github.com/0xPolygonMiden)
- [Miden Discord](https://discord.gg/miden)

---

*Built on Miden Testnet · March 2026*
