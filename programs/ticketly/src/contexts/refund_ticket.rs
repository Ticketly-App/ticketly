use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::EventGateError,
    state::{
        event::EventAccount,
        ticket::{TicketAccount, RefundRecord},
    },
};

#[derive(Accounts)]
pub struct RefundTicket<'info> {
    /// The cancelled event
    #[account(
        seeds = [SEED_EVENT, event.authority.as_ref(), &event.event_id.to_le_bytes()],
        bump  = event.bump,
        has_one = authority @ EventGateError::NotEventAuthority,
        constraint = event.is_cancelled @ EventGateError::EventNotCancelled,
    )]
    pub event: Account<'info, EventAccount>,

    /// The ticket to refund
    #[account(
        seeds = [SEED_TICKET, event.key().as_ref(), &ticket.ticket_number.to_le_bytes()],
        bump  = ticket.bump,
        constraint = ticket.event == event.key() @ EventGateError::TicketEventMismatch,
    )]
    pub ticket: Account<'info, TicketAccount>,

    /// Refund record PDA — init ensures a ticket can only be refunded once
    #[account(
        init,
        payer  = authority,
        space  = RefundRecord::LEN,
        seeds  = [SEED_REFUND, ticket.key().as_ref()],
        bump,
    )]
    pub refund_record: Account<'info, RefundRecord>,

    /// CHECK: Validated to match ticket.owner — receives the refund SOL
    #[account(
        mut,
        constraint = ticket_owner.key() == ticket.owner @ EventGateError::NotTicketOwner,
    )]
    pub ticket_owner: UncheckedAccount<'info>,

    /// Event organizer who pays the refund
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
