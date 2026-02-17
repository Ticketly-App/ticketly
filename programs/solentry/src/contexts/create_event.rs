use anchor_lang::prelude::*;
use crate::{
    constants::*,
    state::{event::EventAccount, organizer::OrganizerProfile},
    instructions::create_event::CreateEventParams,
};

#[derive(Accounts)]
#[instruction(params: CreateEventParams)]
pub struct CreateEvent<'info> {
    #[account(
        init,
        payer  = authority,
        space  = EventAccount::LEN,
        seeds  = [SEED_EVENT, authority.key().as_ref(), &params.event_id.to_le_bytes()],
        bump,
    )]
    pub event: Account<'info, EventAccount>,

    #[account(
        mut,
        seeds  = [b"organizer", authority.key().as_ref()],
        bump   = organizer_profile.bump,
        has_one = authority,
    )]
    pub organizer_profile: Option<Account<'info, OrganizerProfile>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}