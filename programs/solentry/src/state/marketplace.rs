use anchor_lang::prelude::*;

#[account]
pub struct Marketplace {
    pub id: u64,
    pub event: Pubkey,
    pub ticket: Pubkey,
    pub seller: Pubkey,
    pub price: u64,
    pub listed_at: u64,
    pub bump: u8,
}