use anchor_lang::prelude::*;
use crate::{
    contexts::{UpdateEvent, CancelEvent},
    errors::EventGateError,
    events::{EventUpdated, EventCancelled},
    utils::validation::*,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateEventParams {
    pub name:             Option<String>,
    pub description:      Option<String>,
    pub venue:            Option<String>,
    pub metadata_uri:     Option<String>,
    pub event_start:      Option<i64>,
    pub event_end:        Option<i64>,
    pub resale_allowed:   Option<bool>,
    pub max_resale_price: Option<Option<u64>>,
    pub royalty_bps:      Option<u16>,
}

pub fn update_event_handler(
    ctx: Context<UpdateEvent>,
    params: UpdateEventParams,
) -> Result<()> {
    let event = &mut ctx.accounts.event;
    let clock = Clock::get()?;

    require!(event.is_active, EventGateError::EventNotActive);
    require!(!event.is_cancelled, EventGateError::EventAlreadyCancelled);

    if let Some(name) = params.name {
        require!(name.len() <= crate::constants::MAX_NAME_LEN, EventGateError::NameTooLong);
        event.name = name;
    }
    if let Some(desc) = params.description {
        require!(desc.len() <= crate::constants::MAX_DESC_LEN, EventGateError::DescriptionTooLong);
        event.description = desc;
    }
    if let Some(venue) = params.venue {
        require!(venue.len() <= crate::constants::MAX_VENUE_LEN, EventGateError::VenueTooLong);
        event.venue = venue;
    }
    if let Some(uri) = params.metadata_uri {
        require!(uri.len() <= crate::constants::MAX_URI_LEN, EventGateError::UriTooLong);
        event.metadata_uri = uri;
    }
    if let Some(start) = params.event_start {
        let end = params.event_end.unwrap_or(event.event_end);
        require!(start < end, EventGateError::EndBeforeStart);
        event.event_start = start;
    }
    if let Some(end) = params.event_end {
        require!(end > event.event_start, EventGateError::EndBeforeStart);
        event.event_end = end;
    }
    if let Some(resale) = params.resale_allowed {
        event.resale_allowed = resale;
    }
    if let Some(cap) = params.max_resale_price {
        event.max_resale_price = cap;
    }
    if let Some(bps) = params.royalty_bps {
        validate_royalty(bps)?;
        event.royalty_bps = bps;
    }

    emit!(EventUpdated {
        event_pda: event.key(),
        authority: event.authority,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

pub fn cancel_event_handler(ctx: Context<CancelEvent>) -> Result<()> {
    let event = &mut ctx.accounts.event;
    let clock = Clock::get()?;

    require!(!event.is_cancelled, EventGateError::EventAlreadyCancelled);
    require!(
        event.total_checked_in == 0,
        EventGateError::CannotCancelAfterCheckIn
    );

    event.is_active    = false;
    event.is_cancelled = true;

    emit!(EventCancelled {
        event_pda: event.key(),
        authority: event.authority,
        timestamp: clock.unix_timestamp,
    });

    msg!("Event {} cancelled", event.event_id);
    Ok(())
}