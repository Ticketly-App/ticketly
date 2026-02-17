use anchor_lang::prelude::*;
use crate::{
    errors::EventGateError,
    events::{WhitelistEntryAdded, WhitelistEntryRemoved},
    state::{event::EventAccount, whitelist::WhitelistEntry},
    constants::*,
};


pub fn add_whitelist_handler(
    ctx:        Context<AddWhitelist>,
    wallet:     Pubkey,
    allocation: u8,
) -> Result<()> {
    let clock = Clock::get()?;

    require!(allocation >= 1, EventGateError::InvalidSupply);
    require!(ctx.accounts.event.whitelist_gated, EventGateError::WhitelistNotEnabled);
    require!(!ctx.accounts.event.is_cancelled, EventGateError::EventAlreadyCancelled);

    let entry = &mut ctx.accounts.whitelist_entry;
    entry.event      = ctx.accounts.event.key();
    entry.wallet     = wallet;
    entry.allocation = allocation;
    entry.purchased  = 0;
    entry.added_at   = clock.unix_timestamp;
    entry.bump       = ctx.bumps.whitelist_entry;

    emit!(WhitelistEntryAdded {
        event_pda: ctx.accounts.event.key(),
        wallet,
        allocation,
        timestamp: clock.unix_timestamp,
    });

    msg!("WhitelistAdded wallet={} allocation={}", wallet, allocation);
    Ok(())
}

#[derive(Accounts)]
#[instruction(wallet: Pubkey, allocation: u8)]
pub struct AddWhitelist<'info> {
    #[account(
        seeds = [SEED_EVENT, event.authority.as_ref(), &event.event_id.to_le_bytes()],
        bump  = event.bump,
        has_one = authority @ EventGateError::NotEventAuthority,
    )]
    pub event: Account<'info, EventAccount>,

    #[account(
        init,
        payer  = authority,
        space  = WhitelistEntry::LEN,
        seeds  = [b"whitelist", event.key().as_ref(), wallet.as_ref()],
        bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn remove_whitelist_handler(ctx: Context<RemoveWhitelist>) -> Result<()> {
    let clock = Clock::get()?;
    let wallet = ctx.accounts.whitelist_entry.wallet;

    emit!(WhitelistEntryRemoved {
        event_pda: ctx.accounts.event.key(),
        wallet,
        timestamp: clock.unix_timestamp,
    });

    msg!("WhitelistRemoved wallet={}", wallet);
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveWhitelist<'info> {
    #[account(
        seeds = [SEED_EVENT, event.authority.as_ref(), &event.event_id.to_le_bytes()],
        bump  = event.bump,
        has_one = authority @ EventGateError::NotEventAuthority,
    )]
    pub event: Account<'info, EventAccount>,

    #[account(
        mut,
        seeds  = [b"whitelist", event.key().as_ref(), whitelist_entry.wallet.as_ref()],
        bump   = whitelist_entry.bump,
        close  = authority,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,

    #[account(mut)]
    pub authority: Signer<'info>,
}