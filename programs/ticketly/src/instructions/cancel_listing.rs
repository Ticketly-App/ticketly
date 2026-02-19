use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use crate::{
    contexts::CancelListing,
    events::ListingCancelled,
};

pub fn handler(ctx: Context<CancelListing>) -> Result<()> {
    let clock = Clock::get()?;

    let ticket_key   = ctx.accounts.ticket.key();
    let seller_key   = ctx.accounts.seller.key();
    let event_key    = ctx.accounts.event.key();
    let listing_bump = ctx.accounts.listing.bump;
    let ticket_number = ctx.accounts.ticket.ticket_number;

    let ticket_key_ref = ticket_key.as_ref();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"listing",
        ticket_key_ref,
        &[listing_bump],
    ]];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from:      ctx.accounts.escrow_ata.to_account_info(),
                to:        ctx.accounts.seller_ata.to_account_info(),
                authority: ctx.accounts.listing.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    {
        let ticket = &mut ctx.accounts.ticket;
        ticket.is_listed    = false;
        ticket.listed_price = None;
    }

    emit!(ListingCancelled {
        event_pda:  event_key,
        ticket_pda: ticket_key,
        seller:     seller_key,
        timestamp:  clock.unix_timestamp,
    });

    msg!(
        "ListingCancelled #{} seller={}",
        ticket_number, seller_key
    );

    Ok(())
}