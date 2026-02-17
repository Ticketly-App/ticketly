use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata,
    token::{Token, TokenAccount},
};
use crate::{constants::*, state::{event::EventAccount, ticket::TicketAccount}};

#[derive(Accounts)]
pub struct CheckInTicket<'info> {
    #[account(
        mut,
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
        constraint = ticket.event == event.key()
            @ crate::errors::EventGateError::TicketEventMismatch,
    )]
    pub ticket: Account<'info, TicketAccount>,

    // ATA that the attendee should hold the ticket token in.
    #[account(
        mut,
        constraint = attendee_ata.mint   == ticket.mint
            @ crate::errors::EventGateError::MintMismatch,
        constraint = attendee_ata.owner  == attendee.key()
            @ crate::errors::EventGateError::TokenOwnerMismatch,
        constraint = attendee_ata.amount == 1
            @ crate::errors::EventGateError::ZeroBalance,
    )]
    pub attendee_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    pub attendee: UncheckedAccount<'info>,

    pub gate_operator: Signer<'info>,

    pub token_program:          Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
}