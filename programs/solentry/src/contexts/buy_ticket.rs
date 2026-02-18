use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::{
    constants::*,
    state::{event::EventAccount, ticket::{TicketAccount, ListingAccount}},
};

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(
        seeds = [
            SEED_EVENT,
            event.authority.as_ref(),
            &event.event_id.to_le_bytes(),
        ],
        bump = event.bump,
    )]
    pub event: Account<'info, EventAccount>,

    #[account(
        mut,
        seeds = [
            SEED_TICKET,
            ticket.event.as_ref(),
            &ticket.ticket_number.to_le_bytes(),
        ],
        bump = ticket.bump,
        constraint = ticket.event    == event.key()
            @ crate::errors::EventGateError::TicketEventMismatch,
        constraint = ticket.is_listed
            @ crate::errors::EventGateError::TicketNotListed,
        constraint = ticket.owner   != buyer.key()
            @ crate::errors::EventGateError::BuyerIsOwner,
    )]
    pub ticket: Account<'info, TicketAccount>,

    #[account(
        mut,
        seeds  = [b"listing", ticket.key().as_ref()],
        bump   = listing.bump,
        has_one = ticket @ crate::errors::EventGateError::TicketEventMismatch,
        close  = seller,   // return rent to seller on purchase
    )]
    pub listing: Account<'info, ListingAccount>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint      = mint,
        associated_token::authority = listing,
    )]
    pub escrow_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer                       = buyer,
        associated_token::mint      = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_ata: Account<'info, TokenAccount>,

    /// CHECK: Address is constrained to `listing.seller`; used only as rent recipient/transfer target.
    #[account(mut, address = listing.seller)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: Address is constrained to `event.royalty_receiver`; used only as transfer recipient.
    #[account(mut, address = event.royalty_receiver)]
    pub royalty_receiver: UncheckedAccount<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub system_program:           Program<'info, System>,
    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent:                     Sysvar<'info, Rent>,
}