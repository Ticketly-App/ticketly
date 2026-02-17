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
pub struct CancelListing<'info> {
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
        constraint = ticket.is_listed
            @ crate::errors::EventGateError::TicketNotListed,
    )]
    pub ticket: Account<'info, TicketAccount>,

    #[account(
        mut,
        seeds  = [b"listing", ticket.key().as_ref()],
        bump   = listing.bump,
        has_one = seller @ crate::errors::EventGateError::NotTicketOwner,
        close  = seller,
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
        payer                       = seller,
        associated_token::mint      = mint,
        associated_token::authority = seller,
    )]
    pub seller_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub system_program:           Program<'info, System>,
    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent:                     Sysvar<'info, Rent>,
}