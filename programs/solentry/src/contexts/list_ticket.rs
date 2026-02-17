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
pub struct ListTicket<'info> {
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
        constraint = ticket.event  == event.key()
            @ crate::errors::EventGateError::TicketEventMismatch,
        constraint = ticket.owner  == seller.key()
            @ crate::errors::EventGateError::NotTicketOwner,
        constraint = !ticket.is_checked_in
            @ crate::errors::EventGateError::AlreadyCheckedIn,
        constraint = !ticket.is_listed
            @ crate::errors::EventGateError::TicketIsListed,
    )]
    pub ticket: Account<'info, TicketAccount>,

    #[account(
        init,
        payer  = seller,
        space  = ListingAccount::LEN,
        seeds  = [b"listing", ticket.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, ListingAccount>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint      = mint,
        associated_token::authority = seller,
        constraint = seller_ata.amount == 1
            @ crate::errors::EventGateError::ZeroBalance,
    )]
    pub seller_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer                       = seller,
        associated_token::mint      = mint,
        associated_token::authority = listing,
    )]
    pub escrow_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub system_program:           Program<'info, System>,
    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent:                     Sysvar<'info, Rent>,
}