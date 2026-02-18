use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use crate::{
    errors::EventGateError,
    events::PoapMinted,
    state::{event::EventAccount, ticket::TicketAccount, poap::PoapRecord},
    utils::nft::create_ticket_metadata,
    constants::*,
};

pub fn handler(ctx: Context<MintPoap>) -> Result<()> {
    let clock = Clock::get()?;

    {
        let event  = &ctx.accounts.event;
        let ticket = &ctx.accounts.ticket;
        require!(event.poap_enabled,            EventGateError::PoapNotEnabled);
        require!(ticket.is_checked_in,          EventGateError::NotCheckedIn);
        require!(!ticket.poap_minted,           EventGateError::PoapAlreadyMinted);
        require!(ticket.event == event.key(),   EventGateError::TicketEventMismatch);
        require!(ticket.owner == ctx.accounts.holder.key(), EventGateError::NotTicketOwner);
    }

    let event_key      = ctx.accounts.event.key();
    let ticket_key     = ctx.accounts.ticket.key();
    let edition_number = ctx.accounts.event.total_poaps_minted;
    let event_name     = ctx.accounts.event.name.clone();
    let poap_uri       = ctx.accounts.event.poap_metadata_uri.clone();
    let royalty_bps    = ctx.accounts.event.royalty_bps;
    let organiser      = ctx.accounts.event.authority;
    let symbol         = ctx.accounts.event.symbol.clone();
    let poap_bump      = ctx.bumps.poap_record;
    let poap_key       = ctx.accounts.poap_record.key();

    {
        let poap = &mut ctx.accounts.poap_record;
        poap.ticket         = ticket_key;
        poap.event          = event_key;
        poap.mint           = ctx.accounts.poap_mint.key();
        poap.holder         = ctx.accounts.holder.key();
        poap.event_name     = event_name.clone();
        poap.attended_at    = ctx.accounts.ticket.checked_in_at.unwrap_or(clock.unix_timestamp);
        poap.edition_number = edition_number;
        poap.metadata_uri   = poap_uri.clone();
        poap.bump           = poap_bump;
    }

    ctx.accounts.ticket.poap_minted = true;
    ctx.accounts.event.total_poaps_minted += 1;

    let ticket_key_ref = ticket_key.as_ref();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"poap",
        ticket_key_ref,
        &[poap_bump],
    ]];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint:      ctx.accounts.poap_mint.to_account_info(),
                to:        ctx.accounts.holder_ata.to_account_info(),
                authority: ctx.accounts.poap_record.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    let nft_name = format!("{} POAP #{}", event_name, edition_number);
    create_ticket_metadata(
        &ctx.accounts.token_metadata_program,
        &ctx.accounts.poap_metadata_account,
        &ctx.accounts.poap_mint.to_account_info(),
        &ctx.accounts.poap_record.to_account_info(),
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.rent.to_account_info(),
        nft_name,
        symbol,
        poap_uri,
        royalty_bps,
        organiser,
        signer_seeds,
    )?;

    emit!(PoapMinted {
        event_pda:      event_key,
        ticket_pda:     ticket_key,
        poap_pda:       poap_key,
        holder:         ctx.accounts.holder.key(),
        edition_number,
        timestamp:      clock.unix_timestamp,
    });

    msg!("PoapMinted edition={} holder={}", edition_number, ctx.accounts.holder.key());
    Ok(())
}

#[derive(Accounts)]
pub struct MintPoap<'info> {
    #[account(
        mut,
        seeds = [SEED_EVENT, event.authority.as_ref(), &event.event_id.to_le_bytes()],
        bump  = event.bump,
    )]
    pub event: Account<'info, EventAccount>,

    #[account(
        mut,
        seeds = [SEED_TICKET, ticket.event.as_ref(), &ticket.ticket_number.to_le_bytes()],
        bump  = ticket.bump,
        constraint = ticket.event == event.key() @ EventGateError::TicketEventMismatch,
    )]
    pub ticket: Account<'info, TicketAccount>,

    #[account(
        init,
        payer = payer,
        space = PoapRecord::LEN,
        seeds = [b"poap", ticket.key().as_ref()],
        bump,
    )]
    pub poap_record: Account<'info, PoapRecord>,

    #[account(
        init,
        payer             = payer,
        seeds             = [b"poap_mint", ticket.key().as_ref()],
        bump,
        mint::decimals    = 0,
        mint::authority   = poap_record,
        mint::freeze_authority = poap_record,
    )]
    pub poap_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer                       = payer,
        associated_token::mint      = poap_mint,
        associated_token::authority = holder,
    )]
    pub holder_ata: Account<'info, TokenAccount>,

    /// CHECK: Used as metadata account CPI target; validation is performed by Metaplex program during CPI.
    #[account(mut)]
    pub poap_metadata_account: UncheckedAccount<'info>,

    /// CHECK: Address is constrained to `ticket.owner`; used only as ATA authority.
    #[account(constraint = holder.key() == ticket.owner @ EventGateError::NotTicketOwner)]
    pub holder: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program:           Program<'info, System>,
    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program:   Program<'info, Metadata>,
    pub rent:                     Sysvar<'info, Rent>,
}