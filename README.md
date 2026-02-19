# SolEntry

SolEntry is an Anchor-based Solana program for event ticketing with:
- event creation and management
- primary ticket minting
- gate check-in and operator controls
- whitelist allocations
- secondary marketplace listing/buy/cancel
- optional POAP minting for checked-in attendees

## Program IDs
┌─────────┬────────────────────────────────────────────────┐
| Network |  Program ID                                    |
|─────────┼────────────────────────────────────────────────┤
| devnet  | `8QMjo4LvdgEKu67AnE3eGzng5Nc22SrRX7itcvnKqL3W` |
|─────────┼────────────────────────────────────────────────┤
| mainnet | `TBD`                                          |
└─────────┴────────────────────────────────────────────────┘

---

## Repository Structure

```text
SolEntry/
├─ README.md
├─ ARCHITECTURE.md
└─ solentry/
   ├─ Anchor.toml
   ├─ Cargo.toml
   ├─ package.json
   ├─ migrations/
   ├─ programs/
   │  └─ solentry/
   │     └─ src/
   │        ├─ lib.rs
   │        ├─ constants.rs
   │        ├─ errors.rs
   │        ├─ events.rs
   │        ├─ security.rs
   │        ├─ contexts/
   │        ├─ instructions/
   │        ├─ state/
   │        └─ utils/
   └─ tests/
      └─ solentry.ts
```

---

## Tech Stack

- Rust + Anchor (`0.32.1`)
- Solana programs (`cdylib`)
- TypeScript integration tests (`ts-mocha` + `chai`)
- SPL Token + Associated Token Account + Metaplex metadata CPI support

---

## Features

### 1) Platform & Organizer
- Initialize/update protocol platform config (admin, fee bps, receiver, pause flag)
- Initialize/update organizer profile

### 2) Event Lifecycle
- Create event with tier configuration and validation
- Update event metadata and policy fields
- Cancel event (blocked once check-ins exist)
- Withdraw primary-sale SOL from event PDA

### 3) Ticketing
- Mint NFT-like ticket (1 token) per purchase
- Track tier index/type, owner, payment, and lifecycle state
- Transfer ticket peer-to-peer

### 4) Access Control
- Add/remove gate operators per event
- Add/remove whitelist entries with allocation limits
- Enforce check-in window and operator authority

### 5) Marketplace
- List ticket into escrow ATA
- Buy listed ticket with royalty split
- Cancel listing and reclaim token

### 6) POAP
- Mint POAP for checked-in ticket holder when enabled

---

## Program Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for full details.

Key account model (PDAs):
- `platform`: `["platform"]`
- `organizer`: `["organizer", authority]`
- `event`: `["event", authority, event_id_le]`
- `ticket`: `["ticket", event, ticket_number_le]`
- `ticket_mint`: `["ticket_mint", ticket]`
- `listing`: `["listing", ticket]`
- `poap`: `["poap", ticket]`
- `poap_mint`: `["poap_mint", ticket]`
- `whitelist`: `["whitelist", event, wallet]`

---

## Prerequisites

Install:
- Rust toolchain (stable)
- Solana CLI
- Anchor CLI `0.32.1`
- Node.js (LTS recommended)
- Yarn `1.x`

Suggested verification:

```bash
rustc --version
cargo --version
solana --version
anchor --version
node --version
yarn --version
```

---

## Quick Start

From repository root:

```bash
cd solentry
yarn install
anchor build
```

Generate/update IDL + TS types:

```bash
anchor build
```

Artifacts are produced under:
- `target/idl/`
- `target/types/`

---

## Configuration Notes

Current `Anchor.toml` points provider to `devnet`:
- `[provider] cluster = "devnet"`

For local development/tests you can either:
- keep default `anchor test` flow (starts local validator), or
- reuse a running validator with:

```bash
anchor test --skip-local-validator
```

If local validator port conflicts occur (`8899 already in use`), use `--skip-local-validator` or stop stale validator processes.

---

## Build, Deploy, Test

### Build program

```bash
cd solentry
anchor build
```

### Deploy program

```bash
anchor deploy
```

### Run test suite

```bash
anchor test
```

or (when validator is already running):

```bash
anchor test --skip-local-validator
```

Current suite file:
- `solentry/tests/solentry.ts`

---

## Linting / Formatting

```bash
cd solentry
yarn lint
yarn lint:fix
```

---

## Instruction Surface

Entrypoints exposed in `lib.rs`:
- `init_platform`, `update_platform`
- `init_organizer`, `update_organizer`
- `create_event`, `update_event`, `cancel_event`
- `withdraw_revenue`
- `add_operator`, `remove_operator`
- `add_whitelist_entry`, `remove_whitelist_entry`
- `mint_ticket`, `transfer_ticket`, `check_in_ticket`
- `mint_poap`
- `list_ticket`, `buy_ticket`, `cancel_listing`

---

## Security & Validation Highlights

- authority checks for event-admin and gate operations
- event lifecycle checks (active/cancelled)
- timing checks (future start on create, check-in grace window)
- tier bounds and sale-window checks
- whitelist gating and allocation enforcement
- resale cap and royalty constraints
- safe arithmetic guards for lamport math

---

## Events Emitted

Representative events:
- `EventCreated`, `EventUpdated`, `EventCancelled`
- `TicketMinted`, `TicketTransferred`, `TicketCheckedIn`
- `TicketListed`, `TicketSold`, `ListingCancelled`
- `OperatorAdded`, `OperatorRemoved`
- `WhitelistEntryAdded`, `WhitelistEntryRemoved`
- `RevenueWithdrawn`, `PoapMinted`

---

## Known Runtime Warnings (Non-blocking)

During TS tests you may see Node warnings such as:
- `MODULE_TYPELESS_PACKAGE_JSON`
- `DEP0040 punycode`

These are runtime/tooling warnings and do not indicate Solana program logic failure.

---

## Troubleshooting

### `rpc port 8899 is already in use`
- Run with `anchor test --skip-local-validator`, or
- terminate stale validator and rerun tests.

### Test account state collisions on reused validators
- prefer fresh local validator, or
- run deterministic test setup and avoid stale PDA assumptions.

### Metadata CPI on local environments
- if metadata program is unavailable/non-executable locally, core ticket flows can still be validated depending on current program handling.

---

## Development Workflow (Recommended)

1. `anchor build`
2. `anchor test --skip-local-validator` (when validator is already running)
3. review emitted logs/events
4. update IDL/types via rebuild
5. rerun full suite before commit

---

Build with ❤️ for solana