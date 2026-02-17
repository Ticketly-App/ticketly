use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use crate::{
    contexts::ListTicket,
    errors::EventGateError,
    events::TicketListed,
    security::*,
    utils::validation::validate_resale_price,
};

pub fn handler(ctx: Context<ListTicket>, price: u64) -> Result<()> {
    let clock = Clock::get()?;

    {
        let event  = &ctx.accounts.event;
        let ticket = &ctx.accounts.ticket;
        require!(event.resale_allowed, EventGateError::ResaleNotAllowed);
        assert_event_active(event)?;
        assert_ticket_event(ticket, &event.key())?;
        assert_not_checked_in(ticket)?;
        assert_not_listed(ticket)?;
        validate_resale_price(price, event.max_resale_price)?;
    }

    let event_key    = ctx.accounts.event.key();
    let ticket_key   = ctx.accounts.ticket.key();
    let seller_key   = ctx.accounts.seller.key();
    let ticket_number = ctx.accounts.ticket.ticket_number;

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from:      ctx.accounts.seller_ata.to_account_info(),
                to:        ctx.accounts.escrow_ata.to_account_info(),
                authority: ctx.accounts.seller.to_account_info(),
            },
        ),
        1,
    )?;

    {
        let listing = &mut ctx.accounts.listing;
        listing.event      = event_key;
        listing.ticket     = ticket_key;
        listing.seller     = seller_key;
        listing.escrow_ata = ctx.accounts.escrow_ata.key();
        listing.price      = price;
        listing.listed_at  = clock.unix_timestamp;
        listing.bump       = ctx.bumps.listing;
    }

    {
        let ticket = &mut ctx.accounts.ticket;
        ticket.is_listed    = true;
        ticket.listed_price = Some(price);
    }

    emit!(TicketListed {
        event_pda:  event_key,
        ticket_pda: ticket_key,
        seller:     seller_key,
        price,
        timestamp:  clock.unix_timestamp,
    });

    msg!(
        "TicketListed #{} price={} seller={}",
        ticket_number, price, seller_key
    );
    Ok(())
}