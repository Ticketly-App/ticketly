use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::{constants::*, state::{event::EventAccount, ticket::TicketAccount}};

#[derive(Accounts)]
pub struct TransferTicket<'info> {
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
        constraint = ticket.owner  == sender.key()
            @ crate::errors::EventGateError::NotTicketOwner,
    )]
    pub ticket: Account<'info, TicketAccount>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint      = mint,
        associated_token::authority = sender,
        constraint = sender_ata.amount == 1
            @ crate::errors::EventGateError::ZeroBalance,
    )]
    pub sender_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer                       = sender,
        associated_token::mint      = mint,
        associated_token::authority = recipient,
    )]
    pub recipient_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub sender: Signer<'info>,

    /// CHECK: Only used as associated token authority for recipient ATA; no direct data access.
    pub recipient: UncheckedAccount<'info>,

    pub system_program:           Program<'info, System>,
    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent:                     Sysvar<'info, Rent>,
}