use anchor_lang::prelude::*;
use crate::{constants::*, errors::EventGateError, state::event::TicketTier};

pub fn validate_event_strings(
    name:         &str,
    description:  &str,
    venue:        &str,
    metadata_uri: &str,
    symbol:       &str,
) -> Result<()> {
    require!(name.len()         <= MAX_NAME_LEN,   EventGateError::NameTooLong);
    require!(description.len()  <= MAX_DESC_LEN,   EventGateError::DescriptionTooLong);
    require!(venue.len()        <= MAX_VENUE_LEN,  EventGateError::VenueTooLong);
    require!(metadata_uri.len() <= MAX_URI_LEN,    EventGateError::UriTooLong);
    require!(symbol.len()       <= MAX_SYMBOL_LEN, EventGateError::SymbolTooLong);
    Ok(())
}

pub fn validate_event_timing(event_start: i64, event_end: i64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(event_start > now,         EventGateError::StartInPast);
    require!(event_end   > event_start, EventGateError::EndBeforeStart);
    Ok(())
}

pub fn validate_tiers(tiers: &[TicketTier]) -> Result<()> {
    require!(
        !tiers.is_empty() && tiers.len() <= MAX_TIERS as usize,
        EventGateError::InvalidTierCount
    );
    for tier in tiers {
        require!(tier.supply >= 1, EventGateError::InvalidSupply);
        if tier.sale_start != 0 && tier.sale_end != 0 {
            require!(tier.sale_end > tier.sale_start, EventGateError::EndBeforeStart);
        }
    }
    Ok(())
}

pub fn validate_royalty(bps: u16) -> Result<()> {
    require!(bps <= MAX_ROYALTY_BPS, EventGateError::InvalidRoyalty);
    Ok(())
}

pub fn validate_ticket_uri(uri: &str) -> Result<()> {
    require!(uri.len() <= MAX_URI_LEN, EventGateError::UriTooLong);
    Ok(())
}

pub fn validate_resale_price(price: u64, cap: Option<u64>) -> Result<()> {
    require!(price > 0, EventGateError::InvalidPrice);
    if let Some(max) = cap {
        require!(price <= max, EventGateError::PriceExceedsCap);
    }
    Ok(())
}