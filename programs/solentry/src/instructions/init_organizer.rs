use anchor_lang::prelude::*;
use crate::{
    errors::EventGateError,
    state::organizer::OrganizerProfile,
    constants::*,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitOrganizerParams {
    pub name:     String,
    pub website:  String,
    pub logo_uri: String,
}

pub fn init_organizer_handler(
    ctx:    Context<InitOrganizer>,
    params: InitOrganizerParams,
) -> Result<()> {
    let clock = Clock::get()?;

    require!(params.name.len()     <= MAX_NAME_LEN, EventGateError::NameTooLong);
    require!(params.website.len()  <= 100,          EventGateError::UriTooLong);
    require!(params.logo_uri.len() <= MAX_URI_LEN,  EventGateError::UriTooLong);

    let profile = &mut ctx.accounts.organizer_profile;
    profile.authority       = ctx.accounts.authority.key();
    profile.name            = params.name;
    profile.website         = params.website;
    profile.logo_uri        = params.logo_uri;
    profile.total_events    = 0;
    profile.total_tickets   = 0;
    profile.total_revenue   = 0;
    profile.total_royalties = 0;
    profile.is_verified     = false;
    profile.created_at      = clock.unix_timestamp;
    profile.bump            = ctx.bumps.organizer_profile;

    msg!("OrganizerProfile created for {}", ctx.accounts.authority.key());
    Ok(())
}

pub fn update_organizer_handler(
    ctx:    Context<UpdateOrganizer>,
    params: InitOrganizerParams,
) -> Result<()> {
    require!(params.name.len()     <= MAX_NAME_LEN, EventGateError::NameTooLong);
    require!(params.website.len()  <= 100,          EventGateError::UriTooLong);
    require!(params.logo_uri.len() <= MAX_URI_LEN,  EventGateError::UriTooLong);

    let profile = &mut ctx.accounts.organizer_profile;
    profile.name     = params.name;
    profile.website  = params.website;
    profile.logo_uri = params.logo_uri;

    Ok(())
}

#[derive(Accounts)]
pub struct InitOrganizer<'info> {
    #[account(
        init,
        payer  = authority,
        space  = OrganizerProfile::LEN,
        seeds  = [b"organizer", authority.key().as_ref()],
        bump,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateOrganizer<'info> {
    #[account(
        mut,
        seeds  = [b"organizer", authority.key().as_ref()],
        bump   = organizer_profile.bump,
        has_one = authority @ EventGateError::NotEventAuthority,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,

    pub authority: Signer<'info>,
}