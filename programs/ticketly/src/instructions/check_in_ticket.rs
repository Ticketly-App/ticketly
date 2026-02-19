use anchor_lang::prelude::*;
use crate::{
    contexts::CheckInTicket,
    errors::EventGateError,
    events::TicketCheckedIn,
    security::*,
    utils::nft::freeze_ticket_metadata,
};

pub fn handler(ctx: Context<CheckInTicket>) -> Result<()> {
    let clock = Clock::get()?;

    {
        let event  = &ctx.accounts.event;
        let ticket = &ctx.accounts.ticket;
        assert_event_active(event)?;
        assert_checkin_window(event)?;
        assert_gate_operator(event, &ctx.accounts.gate_operator.key())?;
        assert_ticket_event(ticket, &event.key())?;
        assert_not_checked_in(ticket)?;
        assert_not_listed(ticket)?;
    }

    let event_key     = ctx.accounts.event.key();
    let ticket_number = ctx.accounts.ticket.ticket_number;
    let tier_index    = ctx.accounts.ticket.tier_index as usize;
    let attendee_key  = ctx.accounts.attendee.key();
    let operator_key  = ctx.accounts.gate_operator.key();
    let ticket_bump   = ctx.accounts.ticket.bump;
    let ticket_number_bytes = ticket_number.to_le_bytes();

    {
        let ticket = &mut ctx.accounts.ticket;
        ticket.is_checked_in = true;
        ticket.checked_in_at = Some(clock.unix_timestamp);
        ticket.checked_in_by = Some(operator_key);
    }

    {
        let event = &mut ctx.accounts.event;
        event.ticket_tiers[tier_index].checked_in += 1;
        event.total_checked_in += 1;
    }

    let signer_seeds: &[&[&[u8]]] = &[&[
        crate::constants::SEED_TICKET,
        event_key.as_ref(),
        ticket_number_bytes.as_ref(),
        &[ticket_bump],
    ]];

    freeze_ticket_metadata(
        &ctx.accounts.token_metadata_program.to_account_info(),
        &ctx.accounts.metadata_account,
        &ctx.accounts.ticket.to_account_info(),
        signer_seeds,
    )?;

    emit!(TicketCheckedIn {
        event_pda:     event_key,
        ticket_pda:    ctx.accounts.ticket.key(),
        attendee:      attendee_key,
        operator:      operator_key,
        ticket_number,
        timestamp:     clock.unix_timestamp,
    });

    msg!(
        "CheckIn ticket=#{} event={} operator={}",
        ticket_number,
        event_key,
        operator_key
    );
    Ok(())
}