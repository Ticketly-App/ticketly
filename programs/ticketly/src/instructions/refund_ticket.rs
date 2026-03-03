use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer as sol_transfer, Transfer as SolTransfer};
use crate::{
    contexts::RefundTicket,
    events::TicketRefunded,
};

/// Refund the primary ticket price to the current ticket holder.
/// Called by the event organizer (authority) after cancelling an event.
/// This sends `ticket.price_paid` (the original tier price) from the organizer
/// to the current `ticket.owner` (not the original buyer).
pub fn handler(ctx: Context<RefundTicket>) -> Result<()> {
    let clock = Clock::get()?;
    let ticket = &ctx.accounts.ticket;

    // Refund the primary ticket price (what was paid at mint)
    let refund_amount = ticket.price_paid;

    // Transfer SOL from organizer to current ticket owner
    if refund_amount > 0 {
        sol_transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                SolTransfer {
                    from: ctx.accounts.authority.to_account_info(),
                    to:   ctx.accounts.ticket_owner.to_account_info(),
                },
            ),
            refund_amount,
        )?;
    }

    // Record refund in PDA (its existence prevents double-refund)
    let refund = &mut ctx.accounts.refund_record;
    refund.ticket      = ticket.key();
    refund.event       = ctx.accounts.event.key();
    refund.owner       = ticket.owner;
    refund.amount      = refund_amount;
    refund.refunded_at = clock.unix_timestamp;
    refund.bump        = ctx.bumps.refund_record;

    emit!(TicketRefunded {
        event_pda:  ctx.accounts.event.key(),
        ticket_pda: ticket.key(),
        owner:      ticket.owner,
        amount:     refund_amount,
        timestamp:  clock.unix_timestamp,
    });

    msg!(
        "TicketRefunded #{} amount={} to={}",
        ticket.ticket_number,
        refund_amount,
        ticket.owner
    );
    Ok(())
}
