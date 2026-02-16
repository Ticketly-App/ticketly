use anchor_lang::prelude::*;

#[account]
pub struct Event {
    pub id: u64,
    pub name: String,
    pub organizer: Pubkey,
    pub mint: Pubkey,
    pub symbol: String,
    pub uri: String,

    pub max_supply: u64,
    pub minted: u64,

    pub start_date: u64,
    pub end_date: u64,

    pub bump: u8,
}