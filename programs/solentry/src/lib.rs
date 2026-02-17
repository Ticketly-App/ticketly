// ─────────────────────────────────────────────────────────────────────────────
//  event_gate — Onchain Luma: tokenised event ticketing on Solana
//
//  Instruction surface (20 instructions):
//
//  Platform
//    init_platform         — one-time platform config singleton
//    update_platform       — change protocol fee / pause
//
//  Organiser profile
//    init_organizer        — create persistent organiser identity
//    update_organizer      — update name / website / logo
//
//  Event management
//    create_event          — mint a new event with up to 5 tiers
//    update_event          — edit mutable event fields
//    cancel_event          — cancel (only if 0 check-ins)
//    withdraw_revenue      — drain primary-sale lamports from event PDA
//
//  Gate operators
//    add_operator          — register a gate staff wallet
//    remove_operator       — revoke a gate staff wallet
//
//  Whitelist
//    add_whitelist_entry   — allowlist a buyer wallet + allocation
//    remove_whitelist_entry— remove allowlist entry
//
//  Ticket lifecycle
//    mint_ticket           — buy & mint a ticket NFT (whitelist-aware)
//    check_in_ticket       — verify & check in attendee; freeze metadata
//    transfer_ticket       — P2P gift / direct transfer
//    mint_poap             — mint POAP NFT after check-in
//
//  Marketplace
//    list_ticket           — list ticket for resale (escrow)
//    buy_ticket            — purchase listed ticket with royalty split
//    cancel_listing        — delist and return token to seller
// ─────────────────────────────────────────────────────────────────────────────

use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod security;

pub mod state;
pub mod utils;
pub mod contexts;
pub mod instructions;

use contexts::*;
use instructions::*;

declare_id!("3FWjsmMG13BLm5KQQsjZ9jozfSQC91gntgDuAMfNnkbJ");

#[program]
pub mod event_gate {
    use super::*;

    pub fn init_platform(
        ctx:              Context<InitPlatform>,
        protocol_fee_bps: u16,
    ) -> Result<()> {
        instructions::init_platform::init_platform_handler(ctx, protocol_fee_bps)
    }

    // Update platform fee rate, receiver, or pause creation.
    pub fn update_platform(
        ctx:              Context<UpdatePlatform>,
        protocol_fee_bps: Option<u16>,
        fee_receiver:     Option<Pubkey>,
        creation_paused:  Option<bool>,
    ) -> Result<()> {
        instructions::init_platform::update_platform_handler(
            ctx, protocol_fee_bps, fee_receiver, creation_paused,
        )
    }

    // Organiser profile 

    /// Create a persistent OrganizerProfile for cross-event identity & stats.
    pub fn init_organizer(
        ctx:    Context<InitOrganizer>,
        params: InitOrganizerParams,
    ) -> Result<()> {
        instructions::init_organizer::init_organizer_handler(ctx, params)
    }

    // Update organiser display name, website, or logo.
    pub fn update_organizer(
        ctx:    Context<UpdateOrganizer>,
        params: InitOrganizerParams,
    ) -> Result<()> {
        instructions::init_organizer::update_organizer_handler(ctx, params)
    }

    // Event management 
    pub fn create_event(
        ctx:    Context<CreateEvent>,
        params: CreateEventParams,
    ) -> Result<()> {
        instructions::create_event::handler(ctx, params)
    }

    pub fn update_event(
        ctx:    Context<UpdateEvent>,
        params: UpdateEventParams,
    ) -> Result<()> {
        update_event_handler(ctx, params)
    }

    // Cancel an event. Only succeeds if no attendee has checked in yet.
    pub fn cancel_event(ctx: Context<CancelEvent>) -> Result<()> {
        cancel_event_handler(ctx)
    }

    // Withdraw SOL that accumulated in the event PDA from primary sales.
    pub fn withdraw_revenue(
        ctx:    Context<WithdrawRevenue>,
        amount: Option<u64>,
    ) -> Result<()> {
        instructions::withdraw_revenue::handler(ctx, amount)
    }

    // Gate operators 

    /// Register a new gate operator wallet (max 10 per event).
    pub fn add_operator(
        ctx:      Context<AddOperator>,
        operator: Pubkey,
    ) -> Result<()> {
        add_operator_handler(ctx, operator)
    }

    // Revoke a previously registered gate operator.
    pub fn remove_operator(
        ctx:      Context<RemoveOperator>,
        operator: Pubkey,
    ) -> Result<()> {
        remove_operator_handler(ctx, operator)
    }

    // Whitelist 

    // Add a wallet to the event whitelist with a ticket allocation.
    pub fn add_whitelist_entry(
        ctx:        Context<AddWhitelist>,
        wallet:     Pubkey,
        allocation: u8,
    ) -> Result<()> {
        add_whitelist_handler(ctx, wallet, allocation)
    }

    // Remove a wallet from the event whitelist (returns rent to authority).
    pub fn remove_whitelist_entry(ctx: Context<RemoveWhitelist>) -> Result<()> {
        remove_whitelist_handler(ctx)
    }

    pub fn mint_ticket(
        ctx:    Context<MintTicket>,
        params: MintTicketParams,
    ) -> Result<()> {
        instructions::mint_ticket::handler(ctx, params)
    }

    // Verify and check in an attendee at the gate.

    pub fn check_in_ticket(ctx: Context<CheckInTicket>) -> Result<()> {
        instructions::check_in_ticket::handler(ctx)
    }

    // Direct peer-to-peer ticket transfer (gifting).
    pub fn transfer_ticket(ctx: Context<TransferTicket>) -> Result<()> {
        instructions::transfer_ticket::handler(ctx)
    }

    pub fn mint_poap(ctx: Context<MintPoap>) -> Result<()> {
        instructions::mint_poap::handler(ctx)
    }

    // Marketplace

    // List a ticket for resale at `price` lamports.
    pub fn list_ticket(
        ctx:   Context<ListTicket>,
        price: u64,
    ) -> Result<()> {
        instructions::list_ticket::handler(ctx, price)
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        instructions::buy_ticket::handler(ctx)
    }

    // Cancel a resale listing, returning the token to the seller.
    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        instructions::cancel_listing::handler(ctx)
    }
}