use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata,
    token::{Mint, Token, TokenAccount},
};
use crate::{
    constants::*,
    state::{event::EventAccount, ticket::TicketAccount, whitelist::WhitelistEntry},
    instructions::mint_ticket::MintTicketParams,
};

#[derive(Accounts)]
#[instruction(params: MintTicketParams)]
pub struct MintTicket<'info> {
    #[account(
        mut,
        seeds = [SEED_EVENT, event.authority.as_ref(), &event.event_id.to_le_bytes()],
        bump  = event.bump,
    )]
    pub event: Account<'info, EventAccount>,

    #[account(
        init,
        payer  = payer,
        space  = TicketAccount::LEN,
        seeds  = [SEED_TICKET, event.key().as_ref(), &event.total_minted.to_le_bytes()],
        bump,
    )]
    pub ticket: Account<'info, TicketAccount>,

    #[account(
        init,
        payer             = payer,
        seeds             = [SEED_MINT, ticket.key().as_ref()],
        bump,
        mint::decimals    = 0,
        mint::authority   = ticket,
        mint::freeze_authority = ticket,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer                       = payer,
        associated_token::mint      = mint,
        associated_token::authority = recipient,
    )]
    pub recipient_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"whitelist", event.key().as_ref(), payer.key().as_ref()],
        bump  = whitelist_entry.bump,
    )]
    pub whitelist_entry: Option<Account<'info, WhitelistEntry>>,

    pub recipient: SystemAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program:             Program<'info, System>,
    pub token_program:              Program<'info, Token>,
    pub associated_token_program:   Program<'info, AssociatedToken>,
    pub token_metadata_program:     Program<'info, Metadata>,
    pub rent:                       Sysvar<'info, Rent>,
}