# ticketly Program Architecture

## Overview
ticketly is an Anchor-based Solana program for event lifecycle management, ticketing, gate check-in, whitelist controls, POAP issuance, and secondary-market listings.

Core entrypoint:
- Program: `ticketly`
- Program ID: `GawjtcQFx5cnK24VrDiUhGdg4DZbVGLzsSsd4vbxznfs`

## Repository Layout
- `ticketly/programs/ticketly/src/lib.rs` — instruction entrypoints
- `ticketly/programs/ticketly/src/instructions/*` — business logic handlers
- `ticketly/programs/ticketly/src/contexts/*` — account constraints (Anchor `#[derive(Accounts)]`)
- `ticketly/programs/ticketly/src/state/*` — PDA account schemas
- `ticketly/programs/ticketly/src/utils/*` — validation + NFT metadata helpers
- `ticketly/programs/ticketly/src/security.rs` — reusable authorization/time-window guards
- `ticketly/tests/ticketly.ts` — end-to-end integration suite

## PDA Model
Primary deterministic accounts:
- Platform config: `["platform"]`
- Organizer profile: `["organizer", authority]`
- Event: `["event", authority, event_id_le]`
- Ticket: `["ticket", event, ticket_number_le]`
- Ticket mint: `["ticket_mint", ticket]`
- Listing: `["listing", ticket]`
- POAP record: `["poap", ticket]`
- POAP mint: `["poap_mint", ticket]`
- Whitelist entry: `["whitelist", event, wallet]`

## State Accounts
- `PlatformConfig`:
  - `admin`, `protocol_fee_bps`, `fee_receiver`, `creation_paused`
- `OrganizerProfile`:
  - identity metadata + organizer aggregate counters
- `EventAccount`:
  - metadata, schedule, tier config, mint/check-in/revenue counters, resale policy, operator list, whitelist/POAP config
- `TicketAccount`:
  - ownership, tier reference, mint, check-in/listing/transfer state
- `ListingAccount`:
  - seller, escrow ATA, listing price/time
- `PoapRecord` and `WhitelistEntry`:
  - attendance collectible and gated-sale allocation records

## Instruction Domains
### Platform & Organizer
- `init_platform`, `update_platform`
- `init_organizer`, `update_organizer`

### Event Lifecycle
- `create_event`, `update_event`, `cancel_event`
- `withdraw_revenue`

### Access & Gate Controls
- `add_operator`, `remove_operator`
- `add_whitelist_entry`, `remove_whitelist_entry`
- `check_in_ticket`

### Ticketing
- `mint_ticket`
- `transfer_ticket`
- `mint_poap`

### Marketplace
- `list_ticket`
- `buy_ticket`
- `cancel_listing`

## Runtime Flow (High Level)
1. Platform admin initializes protocol settings.
2. Organizer initializes profile and creates event (tiers, schedule, resale policy).
3. Buyers mint tickets (primary payment goes to event PDA).
4. Gate operator/organizer checks in valid ticket holder.
5. Organizer withdraws accumulated primary revenue.
6. Ticket owner can list/cancel/sell on marketplace.
7. On purchase, royalty is paid to `event.royalty_receiver`, seller receives remaining amount, NFT custody moves from listing escrow ATA to buyer ATA.

## Security & Validation Invariants
- Authority checks:
  - Event authority controls mutable event/admin actions.
  - Gate operator must be authority or in event operator set.
- Time checks:
  - Event creation enforces `event_start > now` and `event_end > event_start`.
  - Check-in window opens `CHECK_IN_GRACE_S` before start.
- Listing & transfer safety:
  - Checked-in tickets cannot be listed/transferred as unrestricted assets.
  - Ownership and token-account mint/owner constraints are enforced in account contexts.
- Monetary bounds:
  - Royalty cap (`MAX_ROYALTY_BPS`) and protocol fee cap (`MAX_PROTOCOL_FEE`).
  - Safe arithmetic helpers used for lamport calculations.

## Events Emitted
The program emits typed events for observability and indexing:
- Event lifecycle (`EventCreated`, `EventUpdated`, `EventCancelled`)
- Ticket lifecycle (`TicketMinted`, `TicketTransferred`, `TicketCheckedIn`)
- Marketplace (`TicketListed`, `TicketSold`, `ListingCancelled`)
- Access control (`OperatorAdded`, `OperatorRemoved`, whitelist events)
- Revenue/POAP (`RevenueWithdrawn`, `PoapMinted`)

## Testing Status
Current integration suite (`ticketly/tests/ticketly.ts`) validates platform, organizer, event lifecycle, mint/transfer/check-in, whitelist, revenue withdrawal, marketplace, and cancellation behavior.

Latest run outcome: **26 passing**.

## Notes
- In localnet environments without Metaplex metadata executable, metadata CPI paths are handled defensively so core ticket workflows continue.
- For stable local runs, `anchor test --skip-local-validator` can be used when an external validator is already running.
