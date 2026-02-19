use anchor_lang::prelude::*;
use crate::{
    contexts::CreateEvent,
    events::EventCreated,
    state::event::{GpsCoords, TicketTier},
    utils::validation::*,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateEventParams {
    pub event_id:           u64,
    pub name:               String,
    pub description:        String,
    pub venue:              String,
    pub metadata_uri:       String,
    pub symbol:             String,
    pub gps:                GpsCoords,
    pub event_start:        i64,
    pub event_end:          i64,
    pub ticket_tiers:       Vec<TicketTier>,
    pub resale_allowed:     bool,
    pub max_resale_price:   Option<u64>,
    pub royalty_bps:        u16,
    pub whitelist_gated:    bool,
    pub poap_enabled:       bool,
    pub poap_metadata_uri:  String,
}

pub fn handler(ctx: Context<CreateEvent>, params: CreateEventParams) -> Result<()> {
    let clock = Clock::get()?;

    validate_event_strings(
        &params.name,
        &params.description,
        &params.venue,
        &params.metadata_uri,
        &params.symbol,
    )?;
    validate_event_timing(params.event_start, params.event_end)?;
    validate_tiers(&params.ticket_tiers)?;
    validate_royalty(params.royalty_bps)?;

    if params.poap_enabled {
        use crate::constants::MAX_URI_LEN;
        use crate::errors::EventGateError;
        require!(
            params.poap_metadata_uri.len() <= MAX_URI_LEN,
            EventGateError::UriTooLong
        );
    }

    let event = &mut ctx.accounts.event;
    event.authority          = ctx.accounts.authority.key();
    event.event_id           = params.event_id;
    event.name               = params.name.clone();
    event.description        = params.description;
    event.venue              = params.venue;
    event.metadata_uri       = params.metadata_uri;
    event.symbol             = params.symbol;
    event.gps                = params.gps;
    event.event_start        = params.event_start;
    event.event_end          = params.event_end;
    event.created_at         = clock.unix_timestamp;
    event.ticket_tiers       = params.ticket_tiers;
    event.total_minted       = 0;
    event.total_checked_in   = 0;
    event.total_revenue      = 0;
    event.resale_allowed     = params.resale_allowed;
    event.max_resale_price   = params.max_resale_price;
    event.royalty_bps        = params.royalty_bps;
    event.royalty_receiver   = ctx.accounts.authority.key();
    event.total_royalties    = 0;
    event.gate_operators     = vec![];
    event.whitelist_gated    = params.whitelist_gated;
    event.poap_enabled       = params.poap_enabled;
    event.poap_metadata_uri  = params.poap_metadata_uri;
    event.total_poaps_minted = 0;
    event.is_active          = true;
    event.is_cancelled       = false;
    event.bump               = ctx.bumps.event;

    if let Some(profile) = ctx.accounts.organizer_profile.as_mut() {
        profile.total_events += 1;
    }

    emit!(EventCreated {
        authority:   event.authority,
        event_pda:   event.key(),
        event_id:    event.event_id,
        name:        params.name,
        event_start: event.event_start,
        event_end:   event.event_end,
    });

    msg!("EventCreated id={} pda={}", event.event_id, event.key());
    Ok(())
}