use anchor_lang::prelude::*;

#[account]
pub struct Ticket {
    pub id: u64,
    pub event: Pubkey,
    pub mint: Pubkey,
    pub owner: Pubkey,

    pub price: u64,

    pub is_transferable: bool,
    pub is_burnable: bool,
    pub is_mintable: bool,
    pub is_burnable: bool,

    pub used: bool,
    pub used_at: u64,

    pub bump: u8,
}