use anchor_lang::prelude::*;
use crate::{
    errors::EventGateError,
    state::{event::EventAccount, ticket::TicketAccount},
};

#[inline]
pub fn assert_event_authority(event: &EventAccount, signer: &Pubkey) -> Result<()> {
    require_keys_eq!(event.authority, *signer, EventGateError::NotEventAuthority);
    Ok(())
}

#[inline]
pub fn assert_gate_operator(event: &EventAccount, signer: &Pubkey) -> Result<()> {
    if event.authority == *signer {
        return Ok(());
    }
    require!(
        event.gate_operators.contains(signer),
        EventGateError::NotGateOperator
    );
    Ok(())
}

#[inline]
pub fn assert_event_active(event: &EventAccount) -> Result<()> {
    require!(!event.is_cancelled, EventGateError::EventAlreadyCancelled);
    require!(event.is_active,     EventGateError::EventNotActive);
    Ok(())
}


#[inline]
pub fn assert_checkin_window(event: &EventAccount) -> Result<()> {
    use crate::constants::CHECK_IN_GRACE_S;
    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= event.event_start.saturating_sub(CHECK_IN_GRACE_S),
        EventGateError::CheckInTooEarly
    );
    require!(now <= event.event_end, EventGateError::EventEnded);
    Ok(())
}


#[inline]
pub fn assert_ticket_owner(ticket: &TicketAccount, signer: &Pubkey) -> Result<()> {
    require_keys_eq!(ticket.owner, *signer, EventGateError::NotTicketOwner);
    Ok(())
}

#[inline]
pub fn assert_not_checked_in(ticket: &TicketAccount) -> Result<()> {
    require!(!ticket.is_checked_in, EventGateError::AlreadyCheckedIn);
    Ok(())
}

#[inline]
pub fn assert_is_checked_in(ticket: &TicketAccount) -> Result<()> {
    require!(ticket.is_checked_in, EventGateError::NotCheckedIn);
    Ok(())
}

#[inline]
pub fn assert_not_listed(ticket: &TicketAccount) -> Result<()> {
    require!(!ticket.is_listed, EventGateError::TicketIsListed);
    Ok(())
}

#[inline]
pub fn assert_ticket_event(ticket: &TicketAccount, event_key: &Pubkey) -> Result<()> {
    require_keys_eq!(ticket.event, *event_key, EventGateError::TicketEventMismatch);
    Ok(())
}


#[inline]
pub fn safe_bps(amount: u64, bps: u16) -> Result<u64> {
    let result = (amount as u128)
        .checked_mul(bps as u128)
        .ok_or(EventGateError::Overflow)?
        .checked_div(10_000)
        .ok_or(EventGateError::Overflow)? as u64;
    Ok(result)
}

#[inline]
pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
    a.checked_sub(b).ok_or_else(|| error!(EventGateError::Overflow))
}

#[inline]
pub fn safe_add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or_else(|| error!(EventGateError::Overflow))
}

#[inline]
pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b).ok_or_else(|| error!(EventGateError::Overflow))
}