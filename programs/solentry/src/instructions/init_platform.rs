use anchor_lang::prelude::*;
use crate::{
    errors::EventGateError,
    state::organizer::PlatformConfig,
};

pub fn init_platform_handler(
    ctx:              Context<InitPlatform>,
    protocol_fee_bps: u16,
) -> Result<()> {
    require!(protocol_fee_bps <= 1000, EventGateError::InvalidRoyalty); // max 10%

    let cfg = &mut ctx.accounts.platform_config;
    cfg.admin            = ctx.accounts.admin.key();
    cfg.protocol_fee_bps = protocol_fee_bps;
    cfg.fee_receiver     = ctx.accounts.admin.key();
    cfg.creation_paused  = false;
    cfg.bump             = ctx.bumps.platform_config;

    msg!("PlatformInitialised fee_bps={}", protocol_fee_bps);
    Ok(())
}

pub fn update_platform_handler(
    ctx:              Context<UpdatePlatform>,
    protocol_fee_bps: Option<u16>,
    fee_receiver:     Option<Pubkey>,
    creation_paused:  Option<bool>,
) -> Result<()> {
    let cfg = &mut ctx.accounts.platform_config;

    if let Some(bps) = protocol_fee_bps {
        require!(bps <= 1000, EventGateError::InvalidRoyalty);
        cfg.protocol_fee_bps = bps;
    }
    if let Some(recv) = fee_receiver {
        cfg.fee_receiver = recv;
    }
    if let Some(paused) = creation_paused {
        cfg.creation_paused = paused;
    }

    msg!("PlatformUpdated");
    Ok(())
}

// ─── Contexts ─────────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitPlatform<'info> {
    #[account(
        init,
        payer  = admin,
        space  = PlatformConfig::LEN,
        seeds  = [b"platform"],
        bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePlatform<'info> {
    #[account(
        mut,
        seeds  = [b"platform"],
        bump   = platform_config.bump,
        has_one = admin @ EventGateError::NotEventAuthority,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    pub admin: Signer<'info>,
}