use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, MintTo};
use crate::{
    contexts::MintTicket,
    errors::EventGateError,
    events::TicketMinted,
    security::*,
    utils::nft::create_ticket_metadata,
    utils::validation::validate_ticket_uri,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MintTicketParams {
    pub tier_index:   u8,
    pub metadata_uri: String,
}

pub fn handler(ctx: Context<MintTicket>, params: MintTicketParams) -> Result<()> {
    let clock = Clock::get()?;
    let now   = clock.unix_timestamp;

    {
        let event = &ctx.accounts.event;
        assert_event_active(event)?;

        require!(
            params.tier_index < event.ticket_tiers.len() as u8,
            EventGateError::InvalidTierIndex
        );

        let tier = &event.ticket_tiers[params.tier_index as usize];
        require!(tier.minted < tier.supply, EventGateError::TierSoldOut);
        require!(tier.is_on_sale, EventGateError::TierNotOnSale);

        if tier.sale_start != 0 {
            require!(now >= tier.sale_start, EventGateError::TierSaleNotStarted);
        }
        if tier.sale_end != 0 {
            require!(now <= tier.sale_end, EventGateError::TierSaleEnded);
        }

        validate_ticket_uri(&params.metadata_uri)?;
    }

    if ctx.accounts.event.whitelist_gated {
        let wl = ctx.accounts.whitelist_entry.as_ref()
            .ok_or(EventGateError::WhitelistEntryRequired)?;
        require!(wl.event   == ctx.accounts.event.key(), EventGateError::WhitelistEventMismatch);
        require!(wl.wallet  == ctx.accounts.payer.key(), EventGateError::NotWhitelisted);
        require!(wl.purchased < wl.allocation,           EventGateError::AllocationExhausted);

        let wl_mut = ctx.accounts.whitelist_entry.as_mut()
            .ok_or(EventGateError::WhitelistEntryRequired)?;
        wl_mut.purchased += 1;
    }

    let tier_index    = params.tier_index as usize;
    let ticket_number = ctx.accounts.event.total_minted;
    let price_paid    = ctx.accounts.event.ticket_tiers[tier_index].price;
    let tier_type     = ctx.accounts.event.ticket_tiers[tier_index].tier_type;
    let event_key     = ctx.accounts.event.key();

    if price_paid > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to:   ctx.accounts.event.to_account_info(),
                },
            ),
            price_paid,
        )?;
    }

    {
        let event = &mut ctx.accounts.event;
        event.ticket_tiers[tier_index].minted += 1;
        event.total_minted  += 1;
        event.total_revenue = event.total_revenue.saturating_add(price_paid);
    }

    {
        let ticket = &mut ctx.accounts.ticket;
        ticket.event              = event_key;
        ticket.mint               = ctx.accounts.mint.key();
        ticket.owner              = ctx.accounts.recipient.key();
        ticket.original_buyer     = ctx.accounts.recipient.key();
        ticket.ticket_number      = ticket_number;
        ticket.tier_index         = params.tier_index;
        ticket.tier_type          = tier_type;
        ticket.price_paid         = price_paid;
        ticket.metadata_uri       = params.metadata_uri.clone();
        ticket.is_checked_in      = false;
        ticket.checked_in_at      = None;
        ticket.checked_in_by      = None;
        ticket.poap_minted        = false;
        ticket.is_listed          = false;
        ticket.listed_price       = None;
        ticket.resale_count       = 0;
        ticket.transfer_count     = 0;
        ticket.minted_at          = clock.unix_timestamp;
        ticket.last_transferred_at = None;
        ticket.bump               = ctx.bumps.ticket;
    }

    // ── Mint 1 ticket token ────────────────────────────────────────────────────
    let ticket_number_bytes = ticket_number.to_le_bytes();
    let ticket_bump         = ctx.accounts.ticket.bump;

    let signer_seeds: &[&[&[u8]]] = &[&[
        crate::constants::SEED_TICKET,
        event_key.as_ref(),
        ticket_number_bytes.as_ref(),
        &[ticket_bump],
    ]];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint:      ctx.accounts.mint.to_account_info(),
                to:        ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.ticket.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    let nft_name = format!(
        "{} - {} #{}",
        ctx.accounts.event.name,
        tier_type.label(),
        ticket_number
    );

    create_ticket_metadata(
        &ctx.accounts.token_metadata_program.to_account_info(),
        &ctx.accounts.metadata_account,
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.ticket.to_account_info(),
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.rent.to_account_info(),
        nft_name,
        ctx.accounts.event.symbol.clone(),
        params.metadata_uri,
        ctx.accounts.event.royalty_bps,
        ctx.accounts.event.authority,
        signer_seeds,
    )?;

    emit!(TicketMinted {
        event_pda:     event_key,
        ticket_pda:    ctx.accounts.ticket.key(),
        mint:          ctx.accounts.mint.key(),
        owner:         ctx.accounts.recipient.key(),
        ticket_number,
        tier_index:    params.tier_index,
        paid_lamports: price_paid,
        timestamp:     clock.unix_timestamp,
    });

    msg!(
        "TicketMinted event={} ticket=#{} tier={} mint={}",
        event_key, ticket_number, tier_type.label(), ctx.accounts.mint.key()
    );
    Ok(())
}