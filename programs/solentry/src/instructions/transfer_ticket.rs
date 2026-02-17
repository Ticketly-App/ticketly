use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use crate::{
    contexts::TransferTicket,
    errors::EventGateError,
    events::TicketTransferred,
    security::*,
};

pub fn handler(ctx: Context<TransferTicket>) -> Result<()> {
    let clock = Clock::get()?;

    {
        let event  = &ctx.accounts.event;
        let ticket = &ctx.accounts.ticket;
        assert_event_active(event)?;
        assert_ticket_event(ticket, &event.key())?;
        assert_not_checked_in(ticket)?;
        assert_not_listed(ticket)?;
    }

    let from_key       = ctx.accounts.sender.key();
    let to_key         = ctx.accounts.recipient.key();
    let event_key      = ctx.accounts.event.key();
    let ticket_number  = ctx.accounts.ticket.ticket_number;
    let new_xfer_cnt   = ctx.accounts.ticket.transfer_count.saturating_add(1);

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from:      ctx.accounts.sender_ata.to_account_info(),
                to:        ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.sender.to_account_info(),
            },
        ),
        1,
    )?;

    {
        let ticket = &mut ctx.accounts.ticket;
        ticket.owner               = to_key;
        ticket.transfer_count      = new_xfer_cnt;
        ticket.last_transferred_at = Some(clock.unix_timestamp);
    }

    emit!(TicketTransferred {
        event_pda:     event_key,
        ticket_pda:    ctx.accounts.ticket.key(),
        from:          from_key,
        to:            to_key,
        ticket_number,
        transfer_count: new_xfer_cnt,
        timestamp:     clock.unix_timestamp,
    });

    msg!(
        "TicketTransferred #{} from={} to={} total_transfers={}",
        ticket_number, from_key, to_key, new_xfer_cnt
    );
    Ok(())
}