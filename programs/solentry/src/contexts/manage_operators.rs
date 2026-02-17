use anchor_lang::prelude::*;
use crate::{constants::*, state::event::EventAccount};

#[derive(Accounts)]
pub struct AddOperator<'info> {
    #[account(
        mut,
        seeds = [
            SEED_EVENT,
            event.authority.as_ref(),
            &event.event_id.to_le_bytes(),
        ],
        bump = event.bump,
        has_one = authority @ crate::errors::EventGateError::NotEventAuthority,
    )]
    pub event: Account<'info, EventAccount>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveOperator<'info> {
    #[account(
        mut,
        seeds = [
            SEED_EVENT,
            event.authority.as_ref(),
            &event.event_id.to_le_bytes(),
        ],
        bump = event.bump,
        has_one = authority @ crate::errors::EventGateError::NotEventAuthority,
    )]
    pub event: Account<'info, EventAccount>,

    pub authority: Signer<'info>,
}