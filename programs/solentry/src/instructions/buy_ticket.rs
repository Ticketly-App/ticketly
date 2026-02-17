use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer as sol_transfer, Transfer as SolTransfer};
use anchor_spl::token::{transfer, Transfer};
use crate::{
    contexts::BuyTicket,
    errors::EventGateError,
    events::TicketSold,
    security::*,
};

pub fn handler(ctx: Context<BuyTicket>) -> Result<()> {
    let clock = Clock::get()?;

    {
        let event  = &ctx.accounts.event;
        let ticket = &ctx.accounts.ticket;
        assert_event_active(event)?;
        assert_ticket_event(ticket, &event.key())?;
        assert_not_checked_in(ticket)?;
    }

    let listing_price  = ctx.accounts.listing.price;
    let royalty_bps    = ctx.accounts.event.royalty_bps;
    let event_key      = ctx.accounts.event.key();
    let ticket_key     = ctx.accounts.ticket.key();
    let seller_key     = ctx.accounts.seller.key();
    let buyer_key      = ctx.accounts.buyer.key();
    let ticket_number  = ctx.accounts.ticket.ticket_number;
    let listing_bump   = ctx.accounts.listing.bump;
    let new_resale_cnt = ctx.accounts.ticket.resale_count.saturating_add(1);
    let new_xfer_cnt   = ctx.accounts.ticket.transfer_count.saturating_add(1);

    let royalty_amount = safe_bps(listing_price, royalty_bps)?;
    let seller_amount  = safe_sub(listing_price, royalty_amount)?;

    sol_transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            SolTransfer {
                from: ctx.accounts.buyer.to_account_info(),
                to:   ctx.accounts.seller.to_account_info(),
            },
        ),
        seller_amount,
    )?;

    if royalty_amount > 0 {
        sol_transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                SolTransfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to:   ctx.accounts.royalty_receiver.to_account_info(),
                },
            ),
            royalty_amount,
        )?;
    }

    let ticket_key_bytes = ticket_key.as_ref();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"listing",
        ticket_key_bytes,
        &[listing_bump],
    ]];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from:      ctx.accounts.escrow_ata.to_account_info(),
                to:        ctx.accounts.buyer_ata.to_account_info(),
                authority: ctx.accounts.listing.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    {
        let ticket = &mut ctx.accounts.ticket;
        ticket.owner                = buyer_key;
        ticket.is_listed            = false;
        ticket.listed_price         = None;
        ticket.resale_count         = new_resale_cnt;
        ticket.transfer_count       = new_xfer_cnt;
        ticket.last_transferred_at  = Some(clock.unix_timestamp);
    }

    ctx.accounts.event.total_royalties =
        ctx.accounts.event.total_royalties.saturating_add(royalty_amount);

    emit!(TicketSold {
        event_pda:        event_key,
        ticket_pda:       ticket_key,
        seller:           seller_key,
        buyer:            buyer_key,
        price:            listing_price,
        royalty_lamports: royalty_amount,
        resale_count:     new_resale_cnt,
        timestamp:        clock.unix_timestamp,
    });

    msg!(
        "TicketSold #{} price={} royalty={} resales={}",
        ticket_number, listing_price, royalty_amount, new_resale_cnt
    );
    Ok(())
}